import 'package:flutter/material.dart';
import 'package:wpx/src/rust/api/bridge.dart';

List<double> _niceSegmentLengths(double trackLength) {
  double trackLengthKm = trackLength / 1000;
  Set<double> values = {2, 5, 10};
  if (trackLengthKm > 10) {
    values = {5, 10, 25, 50};
  }
  if (trackLengthKm > 50) {
    values = {10, 25, 50, 75, 100};
  }
  if (trackLengthKm > 100) {
    values = {25, 50, 75, 100, 150, 200};
  }
  if (trackLengthKm > 200) {
    values = {50, 75, 100, 150, 200, 400};
  }
  if (trackLengthKm > 400) {
    values = {100, 150, 200, 300, 600};
  }
  if (trackLengthKm > 600) {
    values = {100, 150, 200, 300, 600, 1000};
  }
  Set<double> s = values.map((e) => e * 1000).toSet();
  double hundredk = 100000;
  double up100 = (trackLength / hundredk).ceil() * hundredk;
  s.add(up100);
  List<double> ret = s.toList();
  ret.sort();
  // if the length is 275 km, the last element will be 300km (and not 400)
  int index = ret.lastIndexWhere((l) => l < trackLength);
  ret.take(index + 2);
  return ret;
}

int _pageCount(
  double trackLength,
  double segmentLengthWithOverlap,
  double segmentOverlap,
) {
  return ((trackLength - segmentOverlap) /
          (segmentLengthWithOverlap - segmentOverlap))
      .ceil();
}

class PageCountInfo {
  double trackLength = 0;
  double segmentLengthWithOverlap = 0;

  int _index = 0;

  List<int> possiblePageCounts = [];

  bool initialized() {
    return trackLength > 0;
  }

  PageCountInfo(this.trackLength, this.segmentLengthWithOverlap) {
    if (trackLength <= 0) {
      return;
    }
    List<double> niceLengths = _niceSegmentLengths(trackLength);
    // for each length, the correspong page count
    Set<int> pageCounts = {};
    for (double niceLength in niceLengths) {
      double withOverlap = niceLength + _segmentOverlap(niceLength);
      int count = _pageCount(
        trackLength,
        withOverlap,
        _segmentOverlap(niceLength),
      );
      debugPrint(
        "print  nice=$niceLength overlap=${_segmentOverlap(niceLength)} count=$count",
      );
      pageCounts.add(count);
    }

    possiblePageCounts = pageCounts.toList();
    possiblePageCounts.sort();
    assert(possiblePageCounts.isNotEmpty);
    _index = 0;
  }

  double getMinIndex() {
    return 0;
  }

  double getMaxIndex() {
    return possiblePageCounts.length - 1;
  }

  double _segmentOverlap(double niceSegmentLength) {
    return (niceSegmentLength / 10).roundToDouble();
  }

  void setNiceSegmentLength(double niceSegmentLength) {
    segmentLengthWithOverlap =
        niceSegmentLength + _segmentOverlap(niceSegmentLength);
    int count = _pageCount(
      trackLength,
      segmentLengthWithOverlap,
      _segmentOverlap(niceSegmentLength),
    );
    assert(possiblePageCounts.contains(count));
    _index = possiblePageCounts.indexWhere((c) => (c == count));
  }

  void setParameters(double length, double overlap) {
    debugPrint("print length=$length overlap=$overlap");
    segmentLengthWithOverlap = length;
    int count = _pageCount(trackLength, segmentLengthWithOverlap, overlap);
    debugPrint("print npages=$count");
    assert(possiblePageCounts.contains(count));
    _index = possiblePageCounts.indexWhere((c) => (c == count));
  }

  int possiblePageIndex() {
    assert(initialized());
    return _index;
  }

  void setPossiblePageIndex(int index) {
    debugPrint("print  setPossiblePageIndex $index");
    _index = index;
    assert(0 <= index && index < possiblePageCounts.length);
    int count = possiblePageCounts[index];
    List<double> niceLengths = _niceSegmentLengths(trackLength);
    double minNiceLength = niceLengths.last;
    for (double niceLength in niceLengths) {
      double withOverlap = niceLength + _segmentOverlap(niceLength);
      int localcount = _pageCount(
        trackLength,
        withOverlap,
        _segmentOverlap(niceLength),
      );
      if (count == localcount) {
        if (minNiceLength > niceLength) {
          minNiceLength = niceLength;
        }
      }
    }
    segmentLengthWithOverlap = minNiceLength + _segmentOverlap(minNiceLength);
  }

  double getSegmentLengthWithOverlap() {
    return segmentLengthWithOverlap;
  }

  double getSegmentOverlap() {
    return (segmentLengthWithOverlap / 11).roundToDouble();
  }

  int getSegmentCount() {
    return possiblePageCounts[_index];
  }

  static String getPagesCountString(int segmentCount) {
    int count = segmentCount;
    String label = "${count * 0.5}";
    if (count % 2 == 0) {
      label = "${(count * 0.5).toInt()}";
    }
    if (count > 2) {
      label = "$label pages";
    } else {
      label = "$label page";
    }
    return label;
  }
}

double segmentLengthWithoutOverlap(Parameters parameter) {
  return parameter.segmentLength - parameter.segmentOverlap;
}
