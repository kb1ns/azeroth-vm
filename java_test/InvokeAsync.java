
public class InvokeAsync {

	public static void main(String[] args) {
		invokeasync(1, 1);
		invokestatic(1, 1);	
	}

	public static void invokestatic(int a, int b) {
		int c = a + b;
	}
	
	public static /** async **/ void invokeasync(int a, int b) {
		int c = a + b;
	}
}
