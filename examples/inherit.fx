class Doughnut -> {
  cook() -> {
    print "Fry until golden brown.";
  }
}

class BostonCream < Doughnut -> {
  cook() -> {
    super.cook();
    print "Pipe full of custard and coat with chocolate.";
  }
}

BostonCream().cook();