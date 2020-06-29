public class Simple {

    public static void test(int a, long b, float c, double d, boolean f) {
        int aa = a + 1;
        long bb = b - 1;
        float cc = c * 1.0f;
	if (f) {
            double dd = d / 1.0d;
	}
    }

    public static void main(String[] args) {
        test(1, 1L, 1.5f, 0.5d, true);
    }
}
