

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

  double project(double value) {
    return getValue(getIndex(value));
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

class SliderValuesWidget extends StatelessWidget {
  final dynamic Function(double) onChanged;
  final String Function(double) formatLabel;
  final SliderValues values;
  const SliderValuesWidget({
    super.key,
    required this.onChanged,
    required this.values,
    required this.formatLabel,
  });

  void onSliderChanged(double sliderIndex) {
    int index = sliderIndex.round();
    onChanged(values.getValue(index));
  }

  int currentWidgetIndex() {
    return values.index();
  }

  @override
  Widget build(BuildContext context) {
    if (values.length()==0) {
      return const Text("loading...");
    }
    String label = formatLabel(values.current());
    return Slider(
      min: 0,
      max: values.length() - 1,
      divisions: values.length() - 1, // not good yet.
      value: currentWidgetIndex().toDouble(),
      label: label,
      onChanged: onSliderChanged,
    );
  }
}