const std = @import("std");

fn fib(n: u32) u32 {
    if (n < 2) {
        return n;
    }
    return fib(n - 2) + fib(n - 1);
}

pub fn main() !void {
    var timer = try std.time.Timer.start();
    const res = fib(40);
    var t = timer.lap();
    t = t / 1000000;
    std.debug.print("Result: {}\n", .{res});
    std.debug.print("Time: {} ms\n", .{t});
}
