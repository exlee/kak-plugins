INPUT_FILE = "cue/LICENSE.cue"
OUTPUT_DIRS = Dir.glob("*/").map { |d| d.chomp("/") }.filter {|d| d != "cue"} + ["."]
TARGETS = OUTPUT_DIRS.map { |dir| "#{dir}/LICENSE" }

task default: TARGETS

TARGETS.each do |target|
  file target => INPUT_FILE do |t|
    sh "cue export #{t.prerequisites.first} --out text -o #{t.name} -e license"
  end
end

