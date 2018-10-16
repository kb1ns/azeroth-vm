
public class HelloWorld {

    static int count = 0;

    static {
        count += 999;
    }

    public static void main(String[] args) {
        System.out.println("Hello, world.");
    }
}
