var before = clock();
fn fib(n) -> {
  if (n < 2) return n;
  return fib(n - 1) + fib(n - 2); 
}
var i = 0;
print fib(15);
var after = clock();
print after-before;