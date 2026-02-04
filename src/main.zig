const std = @import("std");

pub fn main() !void {
    try std.fs.File.stdout().writeAll("Hello, Viewyard!\n");

    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const allocator = arena.allocator();

    var args = try std.process.argsWithAllocator(allocator);
    defer args.deinit();

    std.debug.print("--- Program Arguments ---\n", .{});
    var i: usize = 0;
    while (args.next()) |arg| {
        // 'arg' is a null-terminated slice: [:0]const u8
        std.debug.print("Arg {d}: {s}\n", .{ i, arg });
        i += 1;
    }
    std.debug.print("-------------------------\n", .{});
}
