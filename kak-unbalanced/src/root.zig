const std = @import("std");
const Io = std.Io;

pub const Position = struct {
    line: usize,
    column: usize,
    offset: usize,
    bracket: u8,
};

const ScanState = enum {
    normal,
    double_quote,
    triple_double_quote,
    backtick_fence,
};

pub fn findUnbalanced(
    allocator: std.mem.Allocator,
    source: []const u8,
) ![]Position {
    var stack: std.ArrayList(Position) = .empty;
    defer stack.deinit(allocator);

    var unmatched: std.ArrayList(Position) = .empty;
    errdefer unmatched.deinit(allocator);

    var state: ScanState = .normal;
    var index: usize = 0;
    var line: usize = 1;
    var column: usize = 1;

    while (index < source.len) {
        const remaining = source[index..];

        switch (state) {
            .normal => {
                if (std.mem.startsWith(u8, remaining, "```")) {
                    state = .backtick_fence;
                    advance(source[index .. index + 3], &line, &column);
                    index += 3;
                    continue;
                }
                if (std.mem.startsWith(u8, remaining, "\"\"\"")) {
                    state = .triple_double_quote;
                    advance(source[index .. index + 3], &line, &column);
                    index += 3;
                    continue;
                }

                const byte = source[index];
                switch (byte) {
                    '"' => state = .double_quote,
                    '(', '[', '{' => try stack.append(allocator, .{
                        .line = line,
                        .column = column,
                        .offset = index,
                        .bracket = byte,
                    }),
                    ')', ']', '}' => {
                        if (stack.items.len > 0 and matches(stack.items[stack.items.len - 1].bracket, byte)) {
                            _ = stack.pop();
                        } else {
                            try unmatched.append(allocator, .{
                                .line = line,
                                .column = column,
                                .offset = index,
                                .bracket = byte,
                            });
                        }
                    },
                    else => {},
                }
                advance(source[index .. index + 1], &line, &column);
                index += 1;
            },
            .double_quote => {
                if (source[index] == '\\' and index + 1 < source.len) {
                    advance(source[index .. index + 2], &line, &column);
                    index += 2;
                } else {
                    if (source[index] == '"') state = .normal;
                    advance(source[index .. index + 1], &line, &column);
                    index += 1;
                }
            },
            .triple_double_quote => {
                if (std.mem.startsWith(u8, remaining, "\"\"\"")) {
                    state = .normal;
                    advance(source[index .. index + 3], &line, &column);
                    index += 3;
                } else {
                    advance(source[index .. index + 1], &line, &column);
                    index += 1;
                }
            },
            .backtick_fence => {
                if (std.mem.startsWith(u8, remaining, "```")) {
                    state = .normal;
                    advance(source[index .. index + 3], &line, &column);
                    index += 3;
                } else {
                    advance(source[index .. index + 1], &line, &column);
                    index += 1;
                }
            },
        }
    }

    try unmatched.appendSlice(allocator, stack.items);
    std.mem.sort(Position, unmatched.items, {}, lessThanOffset);
    return unmatched.toOwnedSlice(allocator);
}

pub fn writeKakouneCommands(writer: *Io.Writer, positions: []const Position) Io.Writer.Error!void {
    try writer.writeAll("set-option buffer kak_unbalanced_ranges %val{timestamp}");
    for (positions) |position| {
        try writer.print(" '{d}.{d},{d}.{d}|Unbalanced'", .{
            position.line,
            position.column,
            position.line,
            position.column,
        });
    }
    try writer.writeByte('\n');
}

fn matches(open: u8, close: u8) bool {
    return switch (open) {
        '(' => close == ')',
        '[' => close == ']',
        '{' => close == '}',
        else => false,
    };
}

fn advance(bytes: []const u8, line: *usize, column: *usize) void {
    for (bytes) |byte| {
        if (byte == '\n') {
            line.* += 1;
            column.* = 1;
        } else {
            column.* += 1;
        }
    }
}

fn lessThanOffset(_: void, lhs: Position, rhs: Position) bool {
    return lhs.offset < rhs.offset;
}

test "finds unmatched brackets in source order" {
    const allocator = std.testing.allocator;
    const positions = try findUnbalanced(allocator, "([)]\n}");
    defer allocator.free(positions);

    try std.testing.expectEqual(@as(usize, 3), positions.len);
    try expectPosition(positions[0], 1, 1, '(');
    try expectPosition(positions[1], 1, 3, ')');
    try expectPosition(positions[2], 2, 1, '}');
}

test "ignores brackets inside special markers" {
    const allocator = std.testing.allocator;
    const source =
        \\""" ( [ { """
        \\``` ) ] } ```
    ;
    const positions = try findUnbalanced(allocator, source);
    defer allocator.free(positions);

    try std.testing.expectEqual(@as(usize, 0), positions.len);
}

test "ignores brackets inside double quotes" {
    const allocator = std.testing.allocator;
    const positions = try findUnbalanced(allocator, "\"ignored: ( [ } and escaped quote: \\\"\"");
    defer allocator.free(positions);

    try std.testing.expectEqual(@as(usize, 0), positions.len);
}

test "checks brackets inside single quotes" {
    const allocator = std.testing.allocator;
    const positions = try findUnbalanced(allocator, "'('");
    defer allocator.free(positions);

    try std.testing.expectEqual(@as(usize, 1), positions.len);
    try std.testing.expectEqual(@as(u8, '('), positions[0].bracket);
}

test "checks balanced brackets inside comments" {
    const allocator = std.testing.allocator;
    const source =
        \\fn balanced() void {
        \\    // It's useful to check examples (even in comments).
        \\    /* This block's delimiters are checked too: [ok]. */
        \\}
    ;
    const positions = try findUnbalanced(allocator, source);
    defer allocator.free(positions);

    try std.testing.expectEqual(@as(usize, 0), positions.len);
}

test "reports unbalanced brackets inside comments" {
    const allocator = std.testing.allocator;
    const positions = try findUnbalanced(allocator, "// unbalanced example: (");
    defer allocator.free(positions);

    try std.testing.expectEqual(@as(usize, 1), positions.len);
    try expectPosition(positions[0], 1, 24, '(');
}

test "tracks line and byte column" {
    const allocator = std.testing.allocator;
    const positions = try findUnbalanced(allocator, "ok\n  [");
    defer allocator.free(positions);

    try std.testing.expectEqual(@as(usize, 1), positions.len);
    try expectPosition(positions[0], 2, 3, '[');
}

test "writes Kakoune range updates" {
    var buffer: [512]u8 = undefined;
    var writer: Io.Writer = .fixed(&buffer);

    try writeKakouneCommands(&writer, &.{.{
        .line = 2,
        .column = 3,
        .offset = 0,
        .bracket = '(',
    }});

    try std.testing.expectEqualStrings(
        \\set-option buffer kak_unbalanced_ranges %val{timestamp} '2.3,2.3|Unbalanced'
        \\
    , writer.buffered());
}

fn expectPosition(position: Position, line: usize, column: usize, bracket: u8) !void {
    try std.testing.expectEqual(line, position.line);
    try std.testing.expectEqual(column, position.column);
    try std.testing.expectEqual(bracket, position.bracket);
}
