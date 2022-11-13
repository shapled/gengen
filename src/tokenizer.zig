const std = @import("std");
const Allocator = std.mem.Allocator;
const unicode = std.unicode;

// TODO: implement iterator
pub fn TokenGenerator(comptime T: type) type {
    return struct {
        const Self = @This();

        ptr: *anyopaque = undefined,
        vtable: ?*const VTable = null,

        const VTable = struct {
            next: std.meta.FnPtr(fn (ptr: *anyopaque) ?T),
        };

        pub fn init(
            pointer: anytype,
            nextFn: fn (ptr: @TypeOf(pointer)) ?T,
        ) Self {
            const Ptr = @TypeOf(pointer);
            const ptr_info = @typeInfo(Ptr);

            if (ptr_info != .Pointer) { @compileError("Must be a pointer"); }
            if (ptr_info.Pointer.size != .One) { @compileError("Must be a single-item pointer"); }

            const alignment = ptr_info.Pointer.alignment;

            return .{
                .ptr = pointer,
                .vtable = &.{
                    .next = struct {
                        fn next(ptr: *anyopaque) ?T {
                            const self = @ptrCast(Ptr, @alignCast(alignment, ptr));
                            return @call(.{ .modifier = .always_inline }, nextFn, .{ self });
                        }
                    }.next,
                },
            };
        }

        pub fn next(self: Self) ?T {
            return self.vtable.?.next(self.ptr);
        }
    };
}

pub fn Tokenizer(comptime T: type) type {
    const ArrayList = std.ArrayList(T);

    return struct {
        const Self = @This();

        pos: usize,
        tokens: ArrayList,
        tokengen: TokenGenerator(T),

        pub fn init(tokengen: TokenGenerator(T), allocator: Allocator) Self {
            return .{
                .pos = 0,
                .tokens = ArrayList.init(allocator),
                .tokengen = tokengen,
            };
        }

        pub fn deinit(self: *Self) void {
            self.tokens.deinit();
        }

        pub fn mark(self: *Self) usize {
            return self.pos;
        }

        pub fn reset(self: *Self, pos: usize) void {
            self.pos = pos;
        }

        pub fn get_token(self: *Self) ?T {
            const token = self.peek_token();
            if (token != null) {
                self.pos += 1;
            }
            return token;
        }

        pub fn peek_token(self: *Self) ?T {
            if (self.pos == self.tokens.items.len) {
                const token = self.tokengen.next();
                if (token == null) {
                    return null;
                }
                self.tokens.append(token.?) catch unreachable;
            }
            return self.tokens.items[self.pos];
        }
    };
}

test "TestTokenizerAsLexer" {
    const Input = struct {
        const Self = @This();

        iter: unicode.Utf8Iterator,
        pos: usize,

        fn init(str: []const u8) !Self {
            return .{
                .iter = (try unicode.Utf8View.init(str)).iterator(),
                .pos = 0,
            };
        }

        fn next(self: *Self) ?[]const u8 {
            if (self.iter.nextCodepointSlice()) |cp| {
                return cp;
            }
            return null;
        }
    };

    var input = try Input.init("你a好");
    var tokengen = TokenGenerator([]const u8).init(&input, Input.next);
    var tokenizer = Tokenizer([]const u8).init(tokengen, std.heap.page_allocator);

    const v = tokenizer.mark();
    try std.testing.expectEqual(@as(usize, 0), v);
    try std.testing.expectEqualSlices(u8, @as([]const u8, "你"), tokenizer.peek_token().?);
    try std.testing.expectEqual(@as(usize, 0), tokenizer.mark());
    tokenizer.reset(1);
    try std.testing.expectEqual(@as(usize, 1), tokenizer.mark());
    try std.testing.expectEqualSlices(u8, @as([]const u8, "a"), tokenizer.get_token().?);
    try std.testing.expectEqual(@as(usize, 2), tokenizer.mark());
    try std.testing.expectEqualSlices(u8, @as([]const u8, "好"), tokenizer.get_token().?);
    try std.testing.expectEqual(@as(?[]const u8, null), tokenizer.get_token());
    try std.testing.expectEqual(@as(usize, 3), tokenizer.mark());
    tokenizer.reset(0);
    try std.testing.expectEqual(@as(usize, 0), tokenizer.mark());
}
