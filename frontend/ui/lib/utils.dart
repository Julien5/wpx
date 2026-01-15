import 'dart:math';
import 'dart:ui';

double scaleDown(Size object, Size drawArea) {
  double sw = drawArea.width / object.width;
  double sh = drawArea.height / object.height;
  return [sw, sh, 1.0].reduce(min);
}

List<double> fromKm(List<double> list) {
  List<double> ret = list;
  for (int k = 0; k < list.length; ++k) {
    ret[k] = list[k] * 1000;
  }
  return ret;
}

List<double> fromKmh(List<double> list) {
  List<double> ret = list;
  for (int k = 0; k < list.length; ++k) {
    ret[k] = list[k] * 1000 / 3600;
  }
  return ret;
}

enum ScreenOrientation { desktop, landscape, portrait }

ScreenOrientation screenOrientation(Size size) {
  if (size.width > 1000 && size.height > 500) {
    return ScreenOrientation.desktop;
  }
  if (size.width > size.height) {
    return ScreenOrientation.landscape;
  }
  return ScreenOrientation.portrait;
}

(int, int) sizeAsTuple(Size s) {
  assert(s.width.isFinite);
  assert(s.height.isFinite);
  return (s.width.floor(), s.height.floor());
}

Size makeFinite(Size size) {
  // this size is passed to the backend for rendering
  int max = 1280 * 1280 * 1280;
  int w = max;
  int h = max;
  if (size.width.isFinite) {
    w = size.width.floor();
  }
  if (size.height.isFinite) {
    h = size.height.floor();
  }
  return Size(w.toDouble(), h.toDouble());
}
