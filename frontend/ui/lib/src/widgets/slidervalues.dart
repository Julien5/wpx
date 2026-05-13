import 'dart:async';

import 'package:flutter/material.dart';

class SliderValues {
  List<String> values = [];
  int _index = 0;

  SliderValues();

  void init(List<String> v, String value) {
    values = v;
    _index = getIndex(value);
  }

  void setValue(String value) {
    _index = getIndex(value);
  }

  int getIndex(String value) {
    int ret=values.indexOf(value);
    if (ret<0) {
      debugPrint("could not find $value in {$values}");
      assert(false);
    }
    return ret;
  }

  String getValue(int index) {
    return values[index];
  }

  int length() {
    return values.length;
  }

  String current() {
    if (values.isEmpty) {
      return "";
    }
    return values[_index];
  }

  int index() {
    return _index;
  }
}

class SliderValuesWidget extends StatefulWidget {
  final String Function(String) formatLabel;
  final void Function(String) onValueChanged;
  final bool enabled;
  final int initIndex;
  final List<String> values;
  const SliderValuesWidget({
    super.key,
    required this.values,
    required this.initIndex,
    required this.onValueChanged,
    required this.formatLabel,
    required this.enabled,
  });

  @override
  State<SliderValuesWidget> createState() => _SliderValuesWidgetState();
}

class _SliderValuesWidgetState extends State<SliderValuesWidget> {
  int _currentIndex = 0;
  Timer? _debounceTimer;

  @override
  void initState() {
    super.initState();
    _currentIndex = widget.initIndex;
  }

  void onSliderChanged(double sliderIndex) {
    _debounceTimer?.cancel();
    int index = sliderIndex.round();
    _debounceTimer = Timer(const Duration(milliseconds: 250), () {
      String value = widget.values[index];
      widget.onValueChanged(value);
    });
    setState(() {
      _currentIndex = index;
    });
  }

  @override
  Widget build(BuildContext context) {
    assert(widget.values.isNotEmpty);
  String label="unknown";
  if (_currentIndex>= 0) {
    label = widget.formatLabel(widget.values[_currentIndex]);
  }
    return Slider(
      min: 0,
      max: widget.values.length - 1,
      divisions: widget.values.length - 1,
      value: _currentIndex.toDouble(),
      label: label,
      onChanged: widget.enabled ? onSliderChanged : null,
    );
  }
}
