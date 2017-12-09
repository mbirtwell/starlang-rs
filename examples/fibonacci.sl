function main(argv)
{
    let iterations = 31;

    if len(argv) > 1 {
        let p = parse_int(argv[1]);
        if p[1] {
            println(string_format("Usage: $0 <iterations>", [argv[0]]));
            return 1;
        }
        iterations = p[0];
    }

    let n = new(2);
    let iter = 0;

    n[0] = 0;
    n[1] = 1;

    println(n[0]);

    while iter < iterations {
        println(n[1]);

        let tmp = n[1] + n[0];
        n[0] = n[1];
        n[1] = tmp;

        iter = iter + 1;
    }
}
