import 'package:flutter_test/flutter_test.dart';
import 'package:ui/main.dart';
import 'package:ui/src/rust/frb_generated.dart';
import 'package:integration_test/integration_test.dart';
import 'package:ui/src/rust/api/bridge.dart';

void main() async {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async => await RustLib.init());
  Bridge instance = await Bridge.create();
  testWidgets('Can call rust function', (WidgetTester tester) async {
    await tester.pumpWidget(MyApp(bridge: instance,));
    expect(find.textContaining('Result: `Hello, Tom!`'), findsOneWidget);
  });
}


