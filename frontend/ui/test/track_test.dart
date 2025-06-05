import 'package:flutter_test/flutter_test.dart';
import 'package:ui/src/rust/frb_generated.dart';
import 'package:ui/src/rust/api/frontend.dart';

void test1(Frontend frontend) {}

void main() async {
  test('add() should return the sum of two integers', () {
    int result = 5;
    expect(result, equals(5));
  });
  test("This is async", () async {
    await RustLib.init();
    Frontend instance = await Frontend.create();
    var S = instance.segments();
    expect(S.length, equals(6));
    for(FSegment segment in S) {
      String svg=await instance.renderSegmentTrack(segment: segment);
      expect(svg.length, greaterThan(1000));
    }
    
  }, timeout: Timeout(Duration(seconds: 5)));
}
