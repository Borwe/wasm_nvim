const std = @import("std");
const Alloc = std.mem.Allocator;
const ArrayList = std.ArrayList;
const json = std.json;
const inf = std.builtin.Type;

var gpa = std.heap.GeneralPurposeAllocator(.{}){};

extern fn nvim_echo(start: *const u8, end: *const u8) void;

const LuaTypes = enum(u8) { table, bool, number, empty };

const Param = struct {
    type: u8,
    start: u64,
    end: u64,
};

// generate param field from basic types
// bool, float, double, string
fn getParam(comptime T: type, data: *const T, lua_type: LuaTypes) !Param {
    const size = @sizeOf(T);
    const start = @intFromPtr(data);
    const end = start + size;
    try std.io.getStdOut().writer().print("TYPE: {s}, START: {d}, END: {d} SIZE:{d}\n", .{ @typeName(T), start, end, size });
    return .{ .type = @intFromEnum(lua_type), .start = end, .end = end };
}

// generate param field from a chunk,
// which contains {{ with normal basic types inside}}
// bool, float, double, string
fn getParamFromChunk(comptime T: type, comptime I: type, data: *const ArrayList(ArrayList(T)), comptime innerIsList: bool) !Param {
    const start = @intFromPtr(data);
    var end: u64 = 0;
    var size: u64 = 0;
    if (innerIsList == false) {
        //get end of item
        const items = data.*.items[0].items[0];
        end = start + (@sizeOf(I) * items.len()) + @sizeOf(I);
        size = items.len();
    } else {
        //meaning our inner items is a list
        const items = data.*.items[0].items[0];
        var endPointer: *I = undefined;
        for (items.items) |*i| {
            size += 1;
            endPointer = i;
        }
        end = @intFromPtr(endPointer) + @sizeOf(I);
    }
    try std.io.getStdOut().writer().print("TYPE: {s}, START: {d}, END: {d} SIZE:{d}\n", .{ @typeName(ArrayList(ArrayList(T))), start, end, size });
    return .{ .type = @intFromEnum(LuaTypes.table), .start = end, .end = end };
}

const Functionality = struct {
    name: []const u8, //hold name of function
    params: []const u8, //hold params types, by order
};

fn CreateFunctionality(comptime name: []const u8, comptime params: []const u8) Functionality {
    return .{ .name = name, .params = params };
}

fn genetrateFunctionality(funcs: ArrayList(Functionality)) !ArrayList(u8) {
    var stringified = ArrayList(u8).init(funcs.allocator);
    try json.stringify(funcs.items, .{}, stringified.writer());
    return stringified;
}

export fn alloc(size: u32) *u8 {
    var aloc = gpa.allocator();
    var buf = aloc.alloc(u8, size) catch undefined;
    return &buf[0];
}

export fn dealloc(beg: *[]u8, _: u32) void {
    var aloc = gpa.allocator();
    aloc.free(beg.*);
}

export fn functionality() *const [2]u32 {
    var aloc = gpa.allocator();
    var functions = ArrayList(Functionality).init(aloc);
    _ = functions.append(CreateFunctionality("hi", "void")) catch undefined;
    var funcs_json: ArrayList(u8) = genetrateFunctionality(functions) catch unreachable;
    std.io.getStdOut().writer().print("VAL: {s}\n", .{funcs_json.items}) catch unreachable;
    var results = [_]u32{ @intFromPtr(&funcs_json), funcs_json.items.len };
    return &results;
}

/// All functions that export must have a start and and
/// that allow wasms to reutn pointers to memory
/// where the return value is. returned values would be freed
/// from the wasm_nvim library end
export fn hi() void {
    var aloc = gpa.allocator();
    const stdout = std.io.getStdOut().writer();

    _ = stdout.write("HELLO WASM ZIG\n") catch undefined;

    // create a chunk of strings, which is a {{}}. An Array list in an arraylist
    var arr = ArrayList(ArrayList(ArrayList(u8))).init(aloc);
    var in = ArrayList(ArrayList(u8)).init(aloc);
    var value = ArrayList(u8).init(aloc);
    _ = value.appendSlice("YEAH BABY WASM ZIG IS GANGSTA FOR REAL FOR REAL YO!!!!") catch undefined;
    _ = in.append(value) catch undefined;
    _ = arr.append(in) catch undefined;

    const chunk = getParamFromChunk(ArrayList(u8), u8, &arr, true) catch undefined;
    const hold_message = getParam(bool, &true, LuaTypes.bool) catch undefined;
    const opts = getParam(comptime_int, &0, LuaTypes.empty) catch undefined;
    //const testString = "YOLO";
    //_ = getParam(@TypeOf(testString), &testString, LuaTypes.empty) catch undefined;

    var params = ArrayList(Param).init(aloc);
    //_ = params.append(chunk) catch undefined;
    _ = params.append(chunk) catch undefined;
    _ = params.append(hold_message) catch undefined;
    _ = params.append(opts) catch undefined;

    var return_params = ArrayList(u8).init(aloc);
    //_ = json.stringify(params, .{}, return_params.writer()) catch undefined;

    //pring what would be sent to wasm_nvim side
    _ = stdout.write(return_params.items) catch undefined;
    _ = stdout.write("\n") catch undefined;
}

pub fn main() !void {
    var vals = functionality();
    std.io.getStdOut().writer().print("VAL: {d} {d}\n", .{ vals.*[0], vals.*[1] }) catch unreachable;
    hi();
}
