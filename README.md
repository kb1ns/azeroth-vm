# AzerothVM

JNI is still working in progress, the code snippet below works just fine.

```
public class HelloWorld implements A {

    private int a;

    public void incr() {
        a = a + 1;
    }

    public void decr() {
        a = a - 1;
    }

    static int sum;

    static {
        sum = HelloRust.count;
        for (int i = 0; i < 5; i++) {
            sum += i;
        }
    }

    public static void main(String[] args) {
        HelloWorld s = new HelloWorld();
        s.incr();
        s.decr();
        A a = new HelloRust();
        a.decr();
        HelloRust.say(1, sum);
        int[] array = new int[10];
        int[][] _2darray = new int[10][];
        String[] sarray = new String[10];
        array[0] = 0xcafebabe;
        int tmp = array[0];
    }


    public static class HelloRust extends HelloWorld {

        static int count = 100;

        public void incr() {
        }

        public static int say(int i, int sum) {
            return i + sum;
        }
    }
}

interface A {

    void decr();
}
```

Compile and run

```
cargo build
export JAVA_HOME=${...}
cd java_test && javac *.java
../target/debug/java --classpath . HelloWorld
```
