const std = @import("std");
const ArrayList = std.ArrayList;
const json = std.json;

extern "host" fn get_id() u32;
extern "host" fn set_value(id: u32, loc: u32, size: u32) void;
extern "host" fn get_addr(ptr: *const u8) u32;
extern "host" fn get_value_size(id: u32) u32;
extern "host" fn get_value_addr(id: u32) [*]u8;
extern "host" fn nvim_echo(id: u32) void;
extern "host" fn nvim_create_augroup(id: u32) u32;
extern "host" fn nvim_list_bufs() u32;
extern "host" fn lua_exec(id: u32) void;
extern "host" fn lua_eval(id: u32) u32;

var aloc: std.mem.Allocator = std.heap.page_allocator;

const Variant = union(enum) { I64: i64, String: *[]u8 };

const NvimCreateAutoCmdOpts = struct { group: Variant, pattern: ArrayList(ArrayList(u8)), buffer: ?i64, desc: ?[]u8, callback: ?[]u8, command: ?[]u8, once: ?bool, nested: ?bool };

const NvimCreateAutoCmd = struct { module_from: []u8, events: ArrayList(ArrayList(u8)), opts: NvimCreateAutoCmdOpts };

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
    funcs.append(CreateFunctionality("groups", "void", "void")) catch unreachable;
    funcs.append(CreateFunctionality("consuming", "u32", "void")) catch unreachable;
    funcs.append(CreateFunctionality("returning", "void", "u32")) catch unreachable;
    funcs.append(CreateFunctionality("nvimEcho", "u32", "void")) catch unreachable;
    funcs.append(CreateFunctionality("nvimListBufs", "void", "void")) catch unreachable;
    funcs.append(CreateFunctionality("luaExecExample", "void", "void")) catch unreachable;
    funcs.append(CreateFunctionality("luaEvalExample", "void", "void")) catch unreachable;

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

export fn luaExecExample() void {
    const writer = std.io.getStdOut().writer();
    writer.print("\n--Lua Exec Example--\n", .{}) catch unreachable;
    const script =
        \\local a = 2;
        \\local b = 2;
        \\local c = 2+2;
        \\print("Value of c from lua WASM script is : "..c);
    ;
    const id = get_id();
    set_value(id, get_addr(&script[0]), script.len);
    lua_exec(id);
}

export fn luaEvalExample() void {
    const writer = std.io.getStdOut().writer();
    writer.print("\n--Lua Eval Example--\n", .{}) catch unreachable;
    const script_returns_nil = "require('testing_lua').print_hello_return_nothing()";
    const script_returns_num = "require('testing_lua').print_hello_return_number()";
    var id = get_id();
    set_value(id, get_addr(&script_returns_nil[0]), script_returns_nil.len);
    const return_first = lua_eval(id);

    id = get_id();
    set_value(id, get_addr(&script_returns_num[0]), script_returns_num.len);
    const return_second = lua_eval(id);

    var size_in = get_value_size(return_first);
    var addr_items = get_value_addr(return_first)[0..size_in];
    var return_val_1 = ArrayList(u8).init(aloc);
    defer return_val_1.deinit();
    return_val_1.items = addr_items;
    return_val_1.capacity = size_in;
    writer.print("\n{s} -> Goten the following returned from calling print_hello_return_nothing()", .{return_val_1.items}) catch unreachable;

    size_in = get_value_size(return_second);
    var addr_items_2 = get_value_addr(return_second)[0..size_in];
    var return_val_2 = ArrayList(u8).init(aloc);
    defer return_val_2.deinit();
    return_val_2.items = addr_items_2;
    return_val_2.capacity = size_in;
    writer.print("\n{s} -> Goten the following returned from calling print_hello_return_number()", .{return_val_2.items}) catch unreachable;
}

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
