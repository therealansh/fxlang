class Calc -> {
    init(a,b) -> {
        this.a = a;
        this.b = b;
    }

    add() -> {
        print this.a+this.b;
    }

    subtract() -> {
        print this.a - this.b;
    }

    divide() -> {
        print this.a/this.b;
    }

    multiply() -> {
        print this.a*this.b;
    }
}

print "Enter Value of a and b:";
var a = readNum();
var b = readNum();
var calc = Calc(a,b);
print "Addition:"; calc.add();
print "Subtraction:"; calc.subtract();
print "Multiplication:"; calc.multiply();
print "Division:"; calc.divide();