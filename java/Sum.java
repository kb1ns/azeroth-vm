
public class Sum {

    private int a;

    public void incr() {
        a = a + 1;
    }

    static int sum;

    static {
        sum = HelloWorld.count;
        for (int i = 0; i < 5; i++) {
            sum += i;
        }
    }

    public static void main(String[] args) {
        for (int i = 0; i < 5; i++) {
            new HelloWorld();
            Sum s = new Sum();
            s.incr();
            HelloWorld.say(1, sum);
        }
    }
}
