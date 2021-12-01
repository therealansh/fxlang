class Calc -> {
    add() -> {
        print this.a+this.b;
    }

    multi()->{
        print this.a*this.b;
    }
}

var cal = Calc();
cal.a = 3;
cal.b = 2;
cal.add();
cal.multi();