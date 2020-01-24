
public class Sum {

    static int sum;

    static {
        sum = HelloWorld.count;
        for (int i = 0; i < 5; i++) {
            sum += i;
        }
    }

    public static void main(String[] args) {
        int a = sum;
        for (int i = 0; i < 5; i++) {
	    new HelloWorld();
	    HelloWorld.say(1, sum);
        }
    }
}
