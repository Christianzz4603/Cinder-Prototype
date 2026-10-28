import ghidra.app.decompiler.DecompInterface;
import ghidra.app.decompiler.DecompileResults;
import ghidra.program.model.listing.Function;
import ghidra.program.model.listing.FunctionIterator;
import java.io.PrintWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;

public class ExportPseudocode extends GhidraScript {
    @Override
    protected void run() throws Exception {
        String[] args = getScriptArgs();
        if (args.length < 2) throw new RuntimeException("Usage: <output-dir> <max-functions>");
        Path output = Paths.get(args[0]);
        int limit = Integer.parseInt(args[1]);
        Files.createDirectories(output);
        DecompInterface d = new DecompInterface();
        d.toggleCCode(true);
        d.toggleSyntaxTree(false);
        d.setSimplificationStyle("decompile");
        if (!d.openProgram(currentProgram)) throw new RuntimeException("Decompiler could not open program");
        int tried = 0, ok = 0, failed = 0;
        try (PrintWriter index = new PrintWriter(Files.newBufferedWriter(output.resolve("index.tsv"), StandardCharsets.UTF_8))) {
            index.println("index\taddress\tname\tstatus\tfile");
            FunctionIterator it = currentProgram.getFunctionManager().getFunctions(true);
            while (it.hasNext() && tried < limit && !monitor.isCancelled()) {
                Function fn = it.next();
                if (fn.isExternal()) continue;
                tried++;
                String addr = fn.getEntryPoint().toString().replace(':', '_');
                String name = fn.getName().replaceAll("[^A-Za-z0-9._-]", "_");
                String file = String.format("%06d_%s_%s.c", tried, addr, name);
                DecompileResults result = d.decompileFunction(fn, 60, monitor);
                if (result != null && result.decompileCompleted() && result.getDecompiledFunction() != null) {
                    Files.write(output.resolve(file), result.getDecompiledFunction().getC().getBytes(StandardCharsets.UTF_8));
                    index.println(tried + "\t" + addr + "\t" + name + "\tOK\t" + file);
                    ok++;
                } else {
                    index.println(tried + "\t" + addr + "\t" + name + "\tFAILED\t");
                    failed++;
                }
                if (tried % 25 == 0) println("Progress: " + tried + "/" + limit);
            }
            index.println("# attempted=" + tried + " succeeded=" + ok + " failed=" + failed);
        } finally { d.dispose(); }
        println("Export complete: attempted=" + tried + ", succeeded=" + ok + ", failed=" + failed);
    }
}
