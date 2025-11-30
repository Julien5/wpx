import 'package:ui/src/rust/api/bridge.dart' as bridge;

class SegmentModel extends ChangeNotifier {
  late bridge.Bridge _bridge;
  late Segment _segment;

  SegmentModel(bridge.Bridge bridget, bridge.Segment segment) {
    _bridge = bridge.Bridge.make();
    _segment=segment;
  }

  bridge.Bridge getBridge() {
    return _bridge;
  }
  
  bridge.Parameters parameters() {
    return _bridge.getParameters();
  }

  void setParameters(bridge.Parameters p) {
    _bridge.setParameters(parameters: p);
    notifyListeners();
  }

  Future<List<int>> generateGpx() {
    return _bridge.generateGpx();
  }

  Future<List<int>> generatePdf() {
    return _bridge.generatePdf();
  }

  bridge.SegmentStatistics statistics() {
    return _bridge.statistics();
  }

  List<bridge.Segment> segments() {
    return _bridge.segments();
  }

  bridge.Segment trackSegment() {
    return _bridge.trackSegment();
  }
 
}