# Print primes from 2 to the argument.

function usage(argv) {
    println(string_format("Usage: $0 <max_prime_number>", [argv[0]]));
    exit(1);
}

function main(argv)
{
  if len(argv) != 2 {
    usage(argv);
  }
  let limit_err = parse_int(argv[1]);
  if limit_err[1] {
    usage(argv);
  }
  let limit = limit_err[0];
  let x = 2;
  while x <= limit {
    let y = 2;
    let prime = 1;
    while y * y <= x and prime {
      if x % y == 0 {
        prime = 0;
      }
      y = y + 1;
    }
    if prime {
      println(x);
    }
    x = x + 1;
  }
  return 0;
}
