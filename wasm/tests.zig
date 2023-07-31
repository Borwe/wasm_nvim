const std = @import("std");
const ArrayList = std.ArrayList;
const json = std.json;

extern "host" fn get_id() u32;
extern "host" fn set_value(id: u32, loc: u32, size: u32) void;
extern "host" fn get_addr(ptr: *u8) u32;
extern "host" fn get_value_size(id: u32) u32;
extern "host" fn get_value_addr(id: u32) [*]u8;
extern "host" fn nvim_create_augroup(id: u32) i64;

var aloc: std.mem.Allocator = std.heap.page_allocator;

const Variant = union(enum) { I64: i64, String: *[]u8 };

const NvimCreateGroup = struct { name: []const u8, clear: bool };

const NvimCreateAutoCmdOpts = struct { group: Variant, pattern: ArrayList(ArrayList(u8)), buffer: ?i64, desc: ?[]u8, callback: ?[]u8, command: ?[]u8, once: ?bool, nested: ?bool };

const NvimCreateAutoCmd = struct { module_from: []u8, events: ArrayList(ArrayList(u8)), opts: NvimCreateAutoCmdOpts };

const Type = struct {
    type: []const u8,
};

const Functionality = struct {
    name: []const u8, //hold name of function
    params: Type, //hold params types, by order
    returns: Type,
};

fn CreateFunctionality(comptime name: []const u8, comptime params: Type, comptime returns: Type) Functionality {
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
    funcs.append(CreateFunctionality("groups", .{ .type = "void" }, .{ .type = "void" })) catch unreachable;
    funcs.append(CreateFunctionality("print_something", .{ .type = "u32" }, .{ .type = "bool" })) catch unreachable;

    var jsoned = ArrayList(u8).init(aloc);
    std.json.stringify(funcs.items, .{}, jsoned.writer()) catch undefined;
    const id = get_id();
    const addr = get_addr(&jsoned.items[0]);
    set_value(id, addr, jsoned.items.len);
    return id;
}

export fn printSomething(id: u32) bool {
    const size_in = get_value_size(id);
    const addr_items = get_value_addr(id)[0..size_in];
    var json_vals = ArrayList(u8).init(aloc);
    json_vals.items = addr_items;
    json_vals.capacity = size_in;

    return false;
}

export fn groups() void {
    const outWriter = std.io.getStdOut().writer();
    outWriter.print("--GROUPS TESTS--\n", .{}) catch undefined;

    const group = NvimCreateGroup{ .name = "WasmNvimGrp", .clear = true };
    var jsoned_grp = ArrayList(u8).init(aloc);
    json.stringify(group, .{}, jsoned_grp.writer()) catch unreachable;
    var id = get_id();
    var addr = get_addr(&jsoned_grp.items[0]);
    set_value(id, addr, jsoned_grp.items.len);
    var grp_id = nvim_create_augroup(id);
    outWriter.print("GRP ID: {d}\n", .{grp_id}) catch unreachable;
}
