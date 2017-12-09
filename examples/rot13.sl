# Apply rot13 to input.

function main(argv)
{
  let done = 0;
  while not done {
    let c = getc();
    if c == -1 {
      done = 1;
    }
    if not done {
      if c >= 'A' and c <= 'Z' {
        c = 'A' + (c - 'A' + 13) % 26;
      }
      if c >= 'a' and c <= 'z' {
        c = 'a' + (c - 'a' + 13) % 26;
      }
      putc(c);
    }
  }
}
