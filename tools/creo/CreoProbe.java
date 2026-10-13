import com.ptc.cipjava.jxthrowable;
import com.ptc.pfc.pfcAsyncConnection.AsyncConnection;
import com.ptc.pfc.pfcAsyncConnection.pfcAsyncConnection;
import com.ptc.pfc.pfcFeature.Feature;
import com.ptc.pfc.pfcFeature.Features;
import com.ptc.pfc.pfcFeature.FeatureStatus;
import com.ptc.pfc.pfcModel.ImportFeatAttr;
import com.ptc.pfc.pfcModel.IntfStep;
import com.ptc.pfc.pfcModel.OperationType;
import com.ptc.pfc.pfcModel.pfcModel;
import com.ptc.pfc.pfcPart.Part;
import com.ptc.pfc.pfcSession.Session;
import com.ptc.pfc.pfcSolid.MassProperty;
import java.io.PrintStream;
import java.lang.reflect.Field;
import java.nio.charset.StandardCharsets;

/** Independent J-Link observation process; does not create native sketch/hole features. */
public class CreoProbe {
    private static String quote(String value) {
        StringBuilder out = new StringBuilder("\"");
        for (char c : value.toCharArray()) {
            if (c == '"' || c == '\\') out.append('\\').append(c);
            else if (c < 32) out.append(String.format("\\u%04x", (int)c));
            else out.append(c);
        }
        return out.append('"').toString();
    }

    private static void event(String name, String fields) {
        System.out.println("{\"event\":" + quote(name) + ",\"timestamp_ms\":"
                + System.currentTimeMillis() + (fields.isEmpty() ? "" : "," + fields) + "}");
    }

    private static String statusName(FeatureStatus status) throws IllegalAccessException {
        for (Field field : FeatureStatus.class.getFields()) {
            if (field.getType() == FeatureStatus.class) {
                FeatureStatus candidate = (FeatureStatus) field.get(null);
                if (candidate.getValue() == status.getValue()) return field.getName();
            }
        }
        return "UNKNOWN";
    }

    private static void feature(Feature feature) throws jxthrowable, IllegalAccessException {
        FeatureStatus status = feature.GetStatus();
        event("feature", "\"id\":" + feature.GetId()
                + ",\"type_value\":" + feature.GetFeatType().getValue()
                + ",\"type_name\":" + quote(feature.GetFeatTypeName())
                + ",\"status_value\":" + status.getValue()
                + ",\"status_name\":" + quote(statusName(status)));
    }

    public static void main(String[] args) throws Exception {
        System.setOut(new PrintStream(System.out, true, StandardCharsets.UTF_8.name()));
        System.setErr(new PrintStream(System.err, true, StandardCharsets.UTF_8.name()));
        if (args.length != 4) throw new IllegalArgumentException(
                "Usage: CreoProbe <Creo command> <output dir> <input STEP basename> <part name>");
        AsyncConnection connection = null;
        String stage = "load_library";
        boolean success = false;
        try {
            System.loadLibrary("pfcasyncmt");
            stage = "connect";
            event("start", "\"java_version\":" + quote(System.getProperty("java.version")));
            connection = pfcAsyncConnection.AsyncConnection_Start(args[0], null);
            Session session = connection.GetSession();
            stage = "change_directory";
            session.ChangeDirectory(args[1]);
            event("directory", "\"path\":" + quote(session.GetCurrentDirectory()));
            stage = "create_part";
            Part part = session.CreatePart(args[3]);
            stage = "import_step";
            IntfStep source = pfcModel.IntfStep_Create(args[2]);
            ImportFeatAttr attributes = pfcModel.ImportFeatAttr_Create();
            attributes.SetJoinSurfs(true);
            attributes.SetMakeSolid(true);
            attributes.SetOperation(OperationType.ADD_OPERATION);
            part.CreateImportFeat(source, null, attributes);
            stage = "regenerate";
            part.Regenerate(null);
            event("regenerated", "\"completed\":true");
            Features features = part.ListFeaturesByType(true, null);
            event("feature_count", "\"count\":" + (features == null ? 0 : features.getarraysize()));
            if (features != null) for (int i = 0; i < features.getarraysize(); ++i) feature(features.get(i));
            stage = "save";
            part.Save();
            event("saved", "\"file\":" + quote(part.GetFileName()));
            stage = "export";
            part.ExportIntf3D("result.step", com.ptc.pfc.pfcModel.ExportType.EXPORT_STEP, null);
            event("exported", "\"requested_file\":\"result.step\"");
            String modelLengthUnit = "unresolved";
            try {
                modelLengthUnit = part.GetPrincipalUnits().GetUnit(com.ptc.pfc.pfcBase.UnitType.UNIT_LENGTH).GetName();
                event("model_units", "\"length\":" + quote(modelLengthUnit));
            } catch (jxthrowable error) {
                event("model_units", "\"length\":null,\"reason\":" + quote(error.getClass().getName()));
            }
            stage = "mass_property";
            try {
                MassProperty mass = part.GetMassProperty(null);
                // Model units are not silently equated with the input STEP units.
                event("mass_property", "\"available\":true,\"length_unit\":" + quote(modelLengthUnit)
                        + ",\"volume\":" + mass.GetVolume() + ",\"area\":" + mass.GetSurfaceArea());
            } catch (jxthrowable error) {
                event("mass_property", "\"available\":false,\"volume\":null,\"area\":null,\"reason\":"
                        + quote(error.getClass().getName() + ": " + error.getMessage()));
            }
            success = true;
        } catch (Throwable error) {
            event("failure", "\"stage\":" + quote(stage) + ",\"exception\":"
                    + quote(error.getClass().getName()) + ",\"message\":" + quote(String.valueOf(error.getMessage())));
            throw error;
        } finally {
            if (connection != null) {
                try {
                    connection.End();
                    event("connection_ended", "\"completed\":true");
                } catch (Throwable error) {
                    success = false;
                    event("failure", "\"stage\":\"disconnect\",\"exception\":" + quote(error.getClass().getName()));
                }
            }
            event("completion", "\"success\":" + success);
        }
        if (!success) System.exit(1);
    }
}
