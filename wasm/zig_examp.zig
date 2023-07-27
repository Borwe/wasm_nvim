const std = @import("std");
const ArrayList = std.ArrayList;
const json = std.json;

var aloc = std.heap.page_allocator;

extern "host" fn set_value(id: u32, loc: u32, size: u32) void;
extern "host" fn get_id() u32;
extern "host" fn get_addr(addr: *u8) u32;
extern "host" fn nvim_echo(id: u32) void;

const LuaTypes = enum(u8) { table, bool, number, empty };

const Echo = struct {
    chunk: ArrayList(ArrayList(ArrayList(ArrayList(u8)))),
    history: bool,
    opts: ArrayList(ArrayList(u8)),
    pub fn jsonStringify(self: *const Echo, _: json.StringifyOptions, stream: anytype) !void {
        var writer = json.writeStream(stream, @sizeOf(Echo));
        try writer.beginObject();
        try writer.objectField("chunk");
        for (self.*.chunk.items) |*array| {
            try writer.beginArray();
            for (array.*.items) |*arr| {
                try writer.arrayElem();
                try writer.beginArray();
                for (arr.*.items) |*item| {
                    try writer.arrayElem();
                    try writer.emitString(item.*.items);
                }
                try writer.endArray();
            }
            try writer.endArray();
        }

        try writer.objectField("history");
        try writer.emitBool(self.*.history);

        try writer.objectField("opts");
        try writer.beginArray();
        for (self.*.opts.items) |*arr| {
            try writer.emitString(arr.*.items);
        }
        try writer.endArray();

        try writer.endObject();
    }
};

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
    var functions = ArrayList(Functionality).init(aloc);
    _ = functions.append(CreateFunctionality("hi", Type{ .type = "void" }, Type{ .type = "void" })) catch undefined;
    var stringified = ArrayList(u8).init(aloc);
    json.stringify(functions.items, .{}, stringified.writer()) catch undefined;
    var unmanaged = stringified.moveToUnmanaged();
    // get id for setting a value
    const id = get_id();
    const addr = get_addr(&unmanaged.items[0]);
    //set the value to be consumed as a return type of this function
    set_value(id, addr, unmanaged.items.len);
    return id;
}

/// All functions that export must have a start and and
/// that allow wasms to reutn pointers to memory
/// where the return value is. returned values would be freed
/// from the wasm_nvim library end
export fn hi() void {
    // create a chunk of strings, which is a {{}}. An Array list in an arraylist
    var chunk = ArrayList(ArrayList(ArrayList(ArrayList(u8)))).init(aloc);
    var arr = ArrayList(ArrayList(ArrayList(u8))).init(aloc);
    var in = ArrayList(ArrayList(u8)).init(aloc);
    var value = ArrayList(u8).init(aloc);
    var value2 = ArrayList(u8).init(aloc);
    _ = value.appendSlice("YEAH BABY! WASM ZIG IS GANGSTA FOR REAL FOR REAL YO!!!!") catch undefined;
    _ = value2.appendSlice("ErrorMsg") catch undefined;
    _ = in.append(value) catch undefined;
    _ = in.append(value2) catch undefined;
    _ = arr.append(in) catch undefined;
    _ = chunk.append(arr) catch undefined;

    var to_echo = Echo{ .chunk = chunk, .history = true, .opts = ArrayList(ArrayList(u8)).init(aloc) };

    var to_echo_str = ArrayList(u8).init(aloc);
    _ = json.stringify(to_echo, .{}, to_echo_str.writer()) catch unreachable;

    const id = get_id();
    set_value(id, get_addr(&to_echo_str.items[0]), to_echo_str.items.len);
    nvim_echo(id);
}
