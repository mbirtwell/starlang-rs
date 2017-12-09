function read_two_codes(codes)
{
    let b0 = getc();
    let b1 = getc();
    let b2 = getc();

    if (b0 < 0) { 
        return 0;
    }
    if (b2 < 0) { 
        codes[0] = ((b0 << 8) | b1) & 4095;
        return 1;
    }

    codes[0] = (b0 << 4) | (b1 >> 4);
    codes[1] = ((b1 << 8) | b2) & 4095;

    return 2;
}

# dict: 0-4095: entries, 4096: size
function dict_make()
{
    let d = new(4097);
    let i = 0;
    while i < 256 {
        d[i] = [ i ];
        i = i + 1;
    }
    dict_size_set(d, 256);
    return d;
}

function dict_size(d) { return d[4096]; }
function dict_size_set(d, size) { d[4096] = size; }

function dict_add(d, e)
{
    let size = dict_size(d);
    d[size] = e;
    dict_size_set(d, size + 1);
}

function process_dict()
{
    let dict = dict_make();
    let codes = new(2);
    let old = new(0);
    let count = 0;

    while count < 4096 - 256 {
        let n = read_two_codes(codes);
        let i = 0;

        while i < n {
            let code = codes[i];
            let str = 0;
            let next = 0;

            if (code < dict_size(dict)) {
                str = dict[code];
                next = array_push(old, str[0]);
            }
            if (code >= dict_size(dict)) {
                str = array_push(old, old[0]);
                next = str;
            }

            print(str);

            if (len(old) > 0) {
                dict_add(dict, next);
            }
            old = str;

            i = i + 1;
        }
        count = count + n;

        if n < 2 {
            return 0;
        }
    }
    return 1;
}

function main(argv)
{
    while process_dict() { 
    }
    return 0;
}
