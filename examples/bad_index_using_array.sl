
function f(a, b) {
    return a[b];
}

function main(argv) {
    let a = [1, 2, 3];
    let b = [1];
    return f(a, b);
}