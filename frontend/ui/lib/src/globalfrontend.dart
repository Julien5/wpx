import 'package:ui/src/rust/api/frontend.dart';

class GlobalFrontend {
  // see https://dart.dev/language/constructors#factory-constructors
  static final GlobalFrontend _singleton = GlobalFrontend._internal();
  Frontend? _frontend;
  
  factory GlobalFrontend() {
    return _singleton;
  }

  void setFrontend(Frontend instance) {
    _frontend = instance;
  }

  bool loaded() {
    return _frontend != null;
  }

  Frontend frontend() {
    return _frontend!;
  }
  
  GlobalFrontend._internal();
}