import java.io.FileInputStream;
import java.io.IOException;

public class Load {
    public static class AgentClassLoader extends ClassLoader {
        public Class<?> loadThisClass(byte[] bytes, int length) {
            return defineClass(bytes, 0, length);
        }
    }

    public static void main(String[] args) throws IOException, IllegalAccessException, InstantiationException {
        final byte[] bytes = new byte[10_000_000];
        final int read = new FileInputStream(args[0]).read(bytes);
        new AgentClassLoader().loadThisClass(bytes, read).newInstance().hashCode();
    }
}
