# {{{ assert

function assert(exp)
{
    if exp { return 0; }

    println("Error: assertion failed");
    exit(1);
}

# }}}
# {{{ printing

function println(str)
{
    print(str);
    putc(10);
}

function print(str)
{
    let i = 0;

    if len(str) < 0 {
        # Actually an int
        let digits = new(12);
        let count = 0;
        let num = str;

        if num < 0 {
            digits[0] = '-' - '0';
            num = -num;
            count = count + 1;
        }

        digits[count] = num % 10;
        num = num / 10;
        count = count + 1;

        while num > 0 {
            digits[count] = num % 10;
            num = num / 10;
            count = count + 1;
        }

        while count > 0 {
            putc('0' + digits[count - 1]);
            count = count - 1;
        }
    }
    # else
    while i < len(str) {
        putc(str[i]);
        i = i + 1;
    }
}


# }}}
# {{{ string

function parse_int(str)
{
    let val = 0;
    let negative = 0;
    let pos = 0;
    if str[pos] == '-' {
        negative = 1;
        pos = pos + 1;
    }
    while pos < len(str) and str[pos] >= '0' and str[pos] <= '9' {
        val = val * 10 + str[pos] - '0';
        pos = pos + 1;
    }
    if negative {
        if pos == 1 { return [ 0, 1 ]; }
        return [ -val, 0 ];
    }
    if pos == 0 { return [ 0, 1 ]; }
    return [ val, 0 ];
}

function int_to_string(val)
{
    let result = [];
    if val < 0 {
        result = array_push(result, '-');
        val = -val;
    }
    let x = 10;
    while val >= x {
        x = x * 10;
    }
    while x > 1 {
        x = x / 10;
        result = array_push(result, '0' + val / x);
        val = val % x;
    }
    return result; 
}

function __string_format_helper(dollar, args, result)
{
    let index = parse_int(dollar)[0];
    let val = args[index];
    if len(val) < 0 {
        val = int_to_string(val);
    }
    return array_concat(result, val);
}

function string_format(fmt, args)
{
    let STATE_START = 0;
    let STATE_DOLLAR = 1;

    let i = 0;
    let state = STATE_START;
    let dollar = 0;
    let result = new(0);

    while i < len(fmt) {
        let cont = 1;

        if state == STATE_START {
            let else = 1;
            if fmt[i] == '$' {
                state = STATE_DOLLAR;
                dollar = new(0);
                else = 0;
            }
            if else {
                result = array_push(result, fmt[i]);
            }
            cont = 0;
        }
        if cont and state == STATE_DOLLAR {
            let else = 1;
            if fmt[i] >= '0' and fmt[i] <= '9' {
                dollar = array_push(dollar, fmt[i]);
                else = 0;
            }
            if else {
                if len(dollar) > 0 {
                    result = __string_format_helper(dollar, args, result);
                }
                if fmt[i] == '$' {
                    dollar = new(0);
                    else = 0;
                }
                if else {
                    state = STATE_START;
                    result = array_push(result, fmt[i]);
                }
            }
        }

        i = i + 1;
    }

    if state == STATE_DOLLAR and len(dollar) > 0 {
        result = __string_format_helper(dollar, args, result);
    }

    return result;
}

# }}}
# {{{ array

function array_equal(a, b)
{
    let l = len(a);

    if l != len(b) { return 0; }

    let iter = 0;
    while iter < l {
        if a[iter] != b[iter] { return 0; }
        iter = iter + 1; 
    }
    return 1;
}

function array_copy_region(dst, dst_offset, src, src_offset, n)
{
    assert(n + dst_offset <= len(dst));
    assert(n + src_offset <= len(src));

    let iter = 0;
    while iter < n {
        dst[iter + dst_offset] = src[iter + src_offset];
        iter = iter + 1;
    }
}

function array_copy(dst, src, n)
{
    array_copy_region(dst, 0, src, 0, n);
}

function array_clone(arr)
{
    let n = new(len(arr));
    array_copy(n, arr, len(arr));
    return n;
}

function array_push(arr, item)
{
    let n = new(len(arr) + 1);
    array_copy(n, arr, len(arr));
    n[len(n) - 1] = item;
    return n;
}

function array_concat(arr, items)
{
    let n = new(len(arr) + len(items));
    array_copy_region(n,        0,   arr, 0, len(arr));
    array_copy_region(n, len(arr), items, 0, len(items));
    return n;
}

function array_pop(arr)
{
    if len(arr) == 0 {
        return arr;
    }

    let n = new(len(arr) - 1);
    array_copy(n, arr, len(n));
    return n;
}

function array_shift(arr)
{
    if len(arr) == 0 {
        return arr;
    }

    let n = new(len(arr) - 1);
    array_copy_region(n, 0, arr, 1, len(n));
    return n;
}

function array_head(arr)
{
    assert(len(arr) > 0);
    return arr[0];
}

function array_tail(arr)
{
    assert(len(arr) > 0);
    return arr[len(arr) - 1];
}

# Work In Progress
# linked list
# object:
#   0: size
#   1: head pointer (0 for NULL)
# node:
#   0: value
#   1: next pointer (0 for NULL)

function llist_new()
{
    return [ 0, 0 ];
}

function llist_node_new(item)
{
    return [ item, 0 ];
}

function llist_size(llist)
{
    return llist[0];
}

function llist_push_front(llist, item)
{
    let node = llist_node_new(item);

    if llist[0] == 0 {
        llist[0] = 1;
        llist[1] = node;
        return llist;
    }

    let next = llist[1];
    llist[1] = node;
    node[1] = next;

    llist[0] = llist[0] + 1;

    return llist;
}

function llist_pop_front(llist, item)
{
    if llist[1] == 0 { return llist; }

    llist[1] = llist[1][1];
    llist[0] = llist[0] - 1;

    return llist;
}

function llist_iter_begin(llist)
{
    return llist[1];
}

function llist_iter_next(node)
{
    return node[1];
}

function llist_iter_valid(node)
{
    if len(node) < 0 { return 0; }
    return 1;
}

function llist_iter_value(node)
{
    return node[0];
}

function llist_iter_insert_after(llist, iter, value)
{
    let nnode = llist_node_new(value);
    let tmp = iter[1];
    iter[1] = nnode;
    nnode[1] = tmp;

    llist[0] = llist[0] + 1;
}

function llist_iter_remove_after(llist, iter)
{
    let tmp = iter[1];
    if len(tmp) > 0 {
        iter[1] = tmp[1];
        return 0;
    }
    iter[1] = 0;

    llist[0] = llist[0] - 1;
}
