const std = @import("std");
const Io = std.Io;

const embedded_script = @import("embedded_script");
const kak_unbalanced = @import("kak_unbalanced");

pub fn main(init: std.process.Init) !void {
    const allocator = init.arena.allocator();
    const args = try init.minimal.args.toSlice(allocator);

    var stdout_buffer: [4096]u8 = undefined;
    var stdout_file_writer: Io.File.Writer = .init(.stdout(), init.io, &stdout_buffer);
    const stdout = &stdout_file_writer.interface;

    if (args.len >= 2 and std.mem.eql(u8, args[1], "init")) {
        try stdout.writeAll(embedded_script.content);
        try stdout.flush();
        return;
    }

    const source = try readSource(allocator, init.io, args);
    const positions = try kak_unbalanced.findUnbalanced(allocator, source);

    try kak_unbalanced.writeKakouneCommands(stdout, positions);
    try stdout_file_writer.interface.flush();
}

fn readSource(allocator: std.mem.Allocator, io: Io, args: []const []const u8) ![]const u8 {
    if (args.len >= 2) {
        var file = try Io.Dir.cwd().openFile(io, args[1], .{});
        defer file.close(io);

        var file_buffer: [4096]u8 = undefined;
        var file_reader: Io.File.Reader = .init(file, io, &file_buffer);
        return file_reader.interface.allocRemaining(allocator, .unlimited);
    }

    if (try Io.File.stdin().isTty(io)) return "";

    var stdin_buffer: [4096]u8 = undefined;
    var stdin_reader: Io.File.Reader = .initStreaming(.stdin(), io, &stdin_buffer);
    return stdin_reader.interface.allocRemaining(allocator, .unlimited);
}
