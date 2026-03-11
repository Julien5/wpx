import 'dart:async';

import 'package:flutter/material.dart';

class SliderValues {
  List<double> values = [];
  int _index = 0;

  SliderValues();

  void init(List<double> v, double value) {
    values = v;
    _index = getIndex(value);
  }

  void setValue(double value) {
    _index = getIndex(value);
  }

  int getIndex(double value) {
    int closestIndex = 0;
    double smallestDifference = double.infinity;
    for (int i = 0; i < values.length; i++) {
      double difference = (values[i] - value).abs();
      if (difference < smallestDifference) {
        smallestDifference = difference;
        closestIndex = i;
      }
    }
    return closestIndex;
  }

  double getValue(int index) {
    return values[index];
  }

  int length() {
    return values.length;
  }

  double current() {
    if (values.isEmpty) {
      return 0;
    }
    return values[_index];
  }

  int index() {
    return _index;
  }
}

int getClosestIndex(List<double> values, double value) {
  int closestIndex = 0;
  double smallestDifference = double.infinity;
  for (int i = 0; i < values.length; i++) {
    double difference = (values[i] - value).abs();
    if (difference < smallestDifference) {
      smallestDifference = difference;
      closestIndex = i;
    }
  }
  return closestIndex;
}

class SliderValuesWidget extends StatefulWidget {
  final String Function(double) formatLabel;
  final void Function(double) onValueChanged;
  final bool enabled;
  final int initIndex;
  final List<double> values;
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
      double value = widget.values[index];
      widget.onValueChanged(value);
    });
    setState(() {
      _currentIndex = index;
    });
  }

  @override
  Widget build(BuildContext context) {
    assert(widget.values.isNotEmpty);
    String label = widget.formatLabel(widget.values[_currentIndex]);
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
