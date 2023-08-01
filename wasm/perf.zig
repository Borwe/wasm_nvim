const std = @import("std");
const ArrayList = std.ArrayList;
const json = std.json;

extern "host" fn get_id() u32;
extern "host" fn set_value(id: u32, loc: u32, size: u32) void;
extern "host" fn get_addr(ptr: *u8) u32;
extern "host" fn nvim_create_augroup(id: u32) i64;

var aloc: std.mem.Allocator = std.heap.page_allocator;

const Functionality = struct {
    name: []const u8, //hold name of function
    params: []const u8, //hold params types, by order
    returns: []const u8,
};

fn CreateFunctionality(comptime name: []const u8, comptime params: []const u8, comptime returns: []const u8) Functionality {
    return .{ .name = name, .params = params, .returns = returns };
}

export fn alloc(size: u32) u32 {
    var buf = aloc.alloc(u8, size) catch undefined;
    return get_addr(&buf[0]);
}

export fn dealloc(arr: [*]u8, size: u32) void {
    aloc.free(arr[0..size]);
}

export fn functionality() u32 {
    var funcs = ArrayList(Functionality).init(aloc);
    defer funcs.deinit();
    funcs.append(CreateFunctionality("for_loop", "void", "void")) catch unreachable;

    var jsoned = ArrayList(u8).init(aloc);
    std.json.stringify(funcs.items, .{}, jsoned.writer()) catch undefined;
    const id = get_id();
    const addr = get_addr(&jsoned.items[0]);
    set_value(id, addr, jsoned.items.len);
    return id;
}

export fn for_loop() void {
    var i: u64 = 0;
    var sum: u64 = 0;
    const start = std.time.milliTimestamp();
    while (i < 999999999) {
        sum += i;
        i += 1;
    }
    var diff: f64 = @floatFromInt(std.time.milliTimestamp() - start);
    diff = diff / 1000;

    var writer = std.io.getStdOut().writer();
    writer.print("Wasm Time from inside function For Loop takes: {d}\n", .{diff}) catch undefined;
}
