
public class Sum implements A {

    private int a;

    public void incr() {
        a = a + 1;
    }

    public void decr() {
        a = a - 1;
    }

    static int sum;

    static {
        sum = HelloWorld.count;
        for (int i = 0; i < 5; i++) {
            sum += i;
        }
    }

    public static void main(String[] args) {
            Sum s = new Sum();
            s.incr();
            // s.decr();
            A a = new SumP();
            a.decr();
            HelloWorld.say(1, sum);
    }


    public static class SumP extends Sum {

        public void incr() {
        }
    }

}

interface A {

    void decr();
}
