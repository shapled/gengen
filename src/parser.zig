const std = @import("std");
const Tokenizer = @import("tokenizer.zig").Tokenizer;
const ArrayList = std.ArrayList;
const Allocator = std.mem.Allocator;

const Node = struct {
    const Self = @This();

    kind: []const u8,
    children: ArrayList(Node),

    fn init(allocator: Allocator, kind: []const u8) Self {
        return .{
            .kind = kind,
            .children = ArrayList(Node).init(allocator),
        };
    }

    fn deinit(self: *Self) void {
        self.children.deinit();
    }

    fn equal(self: *const Self, other: *const Node) bool {
        if (!std.mem.eql(u8, self.kind, other.kind)) {
            return false;
        }
        if (self.children.items.len != other.children.items.len) {
            return false;
        }
        var i: usize = 0;
        while (i < self.children.items.len) {
            if (!equal(&self.children.items[i], &other.children.items[i])) {
                return false;
            }
        }
        return true;
    }
};


fn Parser(comptime TokenType: type) type {
    return struct {
        const Self = @This();

        tokenizer: Tokenizer(TokenType),
        tokenEqual: std.meta.FnPtr(fn (t1: TokenType, t2: TokenType) bool),

        fn init(
            tokenizer: Tokenizer(TokenType), 
            comptime tokenEqualFn: fn (t1: TokenType, t2: TokenType) bool,
        ) Self {
            return .{
                .tokenizer = tokenizer,
                .tokenEqual = tokenEqualFn,
            };
        }

        fn mark(self: *Self) usize {
            return self.tokenizer.mark();
        }

        fn reset(self: *Self, pos: usize) void {
            self.tokenizer.reset(pos);
        }

        fn expect(self: *Self, token: TokenType) ?TokenType {
            const next = self.tokenizer.peek_token();
            if (next) |n| {
                if (self.tokenEqual(n, token)) {
                    return self.tokenizer.get_token();
                }
            }
            return null;
        }

        fn loop(
            self: *Self, 
            allocator: Allocator, 
            at_least: usize, 
            at_most: ?usize, 
            args: anytype, 
            func: fn(@TypeOf(args)) ?Node
        ) ?ArrayList(Node) {
            if (at_most) |most| {
                if (most < at_least) {
                    return null;
                }
            }
            const position = self.mark();
            var nodes = ArrayList(Node).init(allocator);
            while (true) {
                if (at_most) |most| {
                    if (nodes.items.len == most) {
                        return nodes;
                    }
                }
                if (func(args)) |node| {
                    nodes.append(node) catch unreachable;
                } else {
                    break;
                }
            }
            if (nodes.items.len >= at_least) {
                return nodes;
            }
            self.reset(position);
            return null;
        }

        fn lookahead(
            self: *Self,
            positive: bool,
            args: anytype, 
            func: fn(@TypeOf(args)) bool
        ) bool {
            const position = self.mark();
            const ret = func(args);
            self.reset(position);
            return positive == ret;
        }
    };
}

test "ParserAsALexerParser" {
    const unicode = @import("std").unicode;
    const TokenGenerator = @import("tokenizer.zig").TokenGenerator;

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

        fn token_equal(t1: []const u8, t2: []const u8) bool {
            return std.mem.eql(u8, t1, t2);
        }
    };

    var input = try Input.init("你abc好");
    const TokenType = []const u8;
    var tokengen = TokenGenerator(TokenType).init(&input, Input.next);
    var tokenizer = Tokenizer(TokenType).init(tokengen, std.heap.page_allocator);
    var parser = Parser(TokenType).init(tokenizer, Input.token_equal);

    try std.testing.expectEqual(@as(usize, 0), parser.mark());
    try std.testing.expectEqual(@as(?TokenType, null), parser.expect("a"));
    try std.testing.expectEqualSlices(u8, @as(TokenType, "你"), parser.expect("你").?);
    try std.testing.expectEqual(@as(usize, 1), parser.mark());

    const is_a = struct {
        fn lambda(str: []const u8) bool {
            return std.mem.eql(u8, str, "a");
        }
    }.lambda;

    const is_not_a = struct {
        fn lambda(str: []const u8) bool {
            return !is_a(str);
        }
    }.lambda;

    try std.testing.expectEqual(true, parser.lookahead(true, @as([]const u8, "a"), is_a));
    try std.testing.expectEqual(true, parser.lookahead(true, @as([]const u8, "a"), is_a));
    try std.testing.expectEqual(true, parser.lookahead(false, @as([]const u8, "a"), is_not_a));
    
    const next_is_a_or_b_or_c = struct {
        fn lambda(p: *Parser(TokenType)) ?Node {
            if ((p.expect("a") != null)
             or (p.expect("b") != null)
             or (p.expect("c") != null)) {
                return Node.init(std.heap.page_allocator, "abc");
            }
            return null;
        }
    }.lambda;

    const abc = Node.init(std.heap.page_allocator, "abc");
    const nodesA = parser.loop(std.heap.page_allocator, 1, null, &parser, next_is_a_or_b_or_c);
    try std.testing.expectEqual(@as(usize, 3), nodesA.?.items.len);
    try std.testing.expectEqual(true, nodesA.?.items[0].equal(&abc));
    try std.testing.expectEqual(true, nodesA.?.items[1].equal(&abc));
    try std.testing.expectEqual(true, nodesA.?.items[2].equal(&abc));
    
    parser.reset(0);
    try std.testing.expectEqual(@as(usize, 0), parser.mark());
}
