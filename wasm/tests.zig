const std = @import("std");
const ArrayList = std.ArrayList;
const json = std.json;

extern "host" fn get_id() u32;
extern "host" fn set_value(id: u32, loc: u32, size: u32) void;
extern "host" fn get_addr(ptr: *u8) u32;
extern "host" fn get_value_size(id: u32) u32;
extern "host" fn get_value_addr(id: u32) [*]u8;
extern "host" fn nvim_echo(id: u32) void;
extern "host" fn nvim_create_augroup(id: u32) u32;
extern "host" fn nvim_list_bufs() u32;

var aloc: std.mem.Allocator = std.heap.page_allocator;

const Variant = union(enum) { I64: i64, String: *[]u8 };

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
    std.io.getStdOut().writer().print("ALLOC SIZE: {d}\n", .{size}) catch undefined;
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
    funcs.append(CreateFunctionality("consuming", .{ .type = "u32" }, .{ .type = "void" })) catch unreachable;
    funcs.append(CreateFunctionality("returning", .{ .type = "void" }, .{ .type = "u32" })) catch unreachable;
    funcs.append(CreateFunctionality("nvimEcho", .{ .type = "u32" }, .{ .type = "void" })) catch unreachable;
    funcs.append(CreateFunctionality("nvimListBufs", .{ .type = "void" }, .{ .type = "void" })) catch unreachable;

    var jsoned = ArrayList(u8).init(aloc);
    std.json.stringify(funcs.items, .{}, jsoned.writer()) catch undefined;
    const id = get_id();
    const addr = get_addr(&jsoned.items[0]);
    set_value(id, addr, jsoned.items.len);
    return id;
}

export fn nvimListBufs() void {
    std.io.getStdOut().writer().print("--NVIM_LIST_BUFS_TEST--", .{}) catch unreachable;
    const id = nvim_list_bufs();
    const size = get_value_size(id);
    var buf_list = get_value_addr(id)[0..size];
    var mng = ArrayList(u8).init(aloc);
    defer mng.deinit();
    mng.items = buf_list;
    mng.capacity = size;

    std.io.getStdOut().writer().print("BUF LIST: {s} \n", .{mng.items}) catch unreachable;
}

export fn nvimEcho(id: u32) void {
    const writer = std.io.getStdOut().writer();
    writer.print("\n--NVIM_ECHO--\n", .{}) catch unreachable;
    nvim_echo(id);
}

export fn consuming(id: u32) void {
    const writer = std.io.getStdOut().writer();
    writer.print("\n--CONSUMING--\n", .{}) catch unreachable;
    const size_in = get_value_size(id);
    const addr_items = get_value_addr(id);
    writer.print("Starting AREA {s}\n", .{addr_items[0..size_in]}) catch unreachable;
}

export fn returning() u32 {
    const writer = std.io.getStdOut().writer();
    writer.print("\n--RETURNING--\n", .{}) catch unreachable;

    var vals = ArrayList(u8).init(aloc);
    vals.appendSlice("{\"yoo\":\"YOLO!!\"}") catch unreachable;
    const id = get_id();
    set_value(id, get_addr(&vals.items[0]), vals.items.len);
    return id;
}

//export fn printSomething(id: u32) u32 {
//    const size_in = get_value_size(id);
//    const addr_items = get_value_addr(id)[0..size_in];
//    var json_vals = ArrayList(u8).init(aloc);
//    json_vals.items = addr_items;
//    json_vals.capacity = size_in;
//
//    std.io.getStdOut().writer().print("WE GOT: {s}", .{json_vals.items}) catch undefined;
//
//    var to_return = ArrayList(u8).init(aloc);
//    to_return.appendSlice("{woot: \"WOOOT WOOT!!\"}") catch unreachable;
//
//    const id_r = get_id();
//    set_value(id, get_addr(&to_return.items[0]), to_return.items.len);
//    return id_r;
//}

export fn groups() void {
    const outWriter = std.io.getStdOut().writer();
    outWriter.print("--GROUPS TESTS--\n", .{}) catch undefined;

    var jsoned_grp = ArrayList(u8).init(aloc);
    jsoned_grp.appendSlice("[\"MyOwnTestGroup\", {\"clear\": false}]") catch unreachable;
    var id = get_id();
    var addr = get_addr(&jsoned_grp.items[0]);
    set_value(id, addr, jsoned_grp.items.len);

    id = nvim_create_augroup(id);
    //read value returned from calling
    var returned = ArrayList(u8).init(aloc);
    defer returned.deinit();
    returned.capacity = get_value_size(id);
    returned.items = get_value_addr(id)[0..returned.capacity];
    outWriter.print("\nGRP ID: {s}\n", .{returned.items}) catch unreachable;
}
