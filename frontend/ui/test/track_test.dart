import 'package:flutter_test/flutter_test.dart';
import 'package:ui/src/rust/frb_generated.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:sprintf/sprintf.dart';

Future<bool> testSegment(Bridge bridge, Segment segment) async {
  String svg = await bridge.renderSegmentTrack(segment: segment);
  print(sprintf("            ID: %d", [segment.id.toInt()]));
  print(sprintf("    svg length: %d bytes", [svg.length]));
  return svg.length > 2000;
}

void main() {
  test('add() should return the sum of two integers', () {
    int result = 5;
    expect(result, equals(5));
  });
  test("This is async", () async {
    await RustLib.init();
    Bridge bridge = await Bridge.create();
    var S = bridge.segments();
    expect(S.length, equals(6));
    for (Segment segment in S) {
      bool ok = await testSegment(bridge,segment);
      expect(ok, true);
    }
  }, timeout: Timeout(Duration(seconds: 5)));
}
