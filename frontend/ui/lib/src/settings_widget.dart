import 'dart:developer' as developer;
import 'dart:math' as math;
import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/routes.dart';

class Selector extends StatelessWidget {
  final String text;
  final double min;
  final double max;
  final double value;
  final Function(double) onChanged;
  const Selector({
    super.key,
    required this.min,
    required this.max,
    required this.text,
    required this.value,
    required this.onChanged,
  });

  @override
  Widget build(BuildContext ctx) {
    return Center(
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Text(text),
          Slider(
            min: min,
            max: max,
            divisions: math.max(
              5,
              (((min - max) / 20).floor() * 20),
            ), // not good yet.
            value: value.clamp(min, max),
            label: text,
            onChanged: onChanged,
          ),
        ],
      ),
    );
  }
}

class SegmentsSettings extends StatefulWidget {
  const SegmentsSettings({super.key});

  @override
  State<SegmentsSettings> createState() => _SegmentsSettingsState();
}

class _SegmentsSettingsState extends State<SegmentsSettings> {
  DateTime selectedTime = DateTime.now();
  double selectedSpeed = 15 * 1000.0 / 3600;
  double selectedSegmentLength = 100000;

  void onDone(BuildContext context) {
    SegmentsProvider provider = Provider.of<SegmentsProvider>(
      context,
      listen: false,
    );
    provider.setStartTime(selectedTime);
    provider.setSpeed(selectedSpeed);
    provider.setSegmentLength(selectedSegmentLength);
    Navigator.of(context).pushNamed(RouteManager.segmentsView);
  }

  Future<void> _selectTime(BuildContext context) async {
    final TimeOfDay? picked = await showTimePicker(
      context: context,
      initialTime: TimeOfDay.now(),
    );

    if (picked != null && picked != selectedTime) {
      DateTime now = DateTime.now();
      DateTime dateTime = DateTime(
        now.year,
        now.month,
        now.day,
        picked.hour,
        picked.minute,
      );
      setState(() {
        selectedTime = dateTime;
      });
    }
  }

  String timeAsString() {
    return DateFormat('dd.MM HH:mm').format(selectedTime);
  }

  String speedAsString() {
    double kmh = selectedSpeed * 3.6;
    return "Speed: ${kmh.toStringAsFixed(1)} kmh";
  }

  String segmentLengthAsString() {
    double km = selectedSegmentLength / 1000;
    return "Segment length: ${km.toStringAsFixed(1)} km";
  }

  @override
  Widget build(BuildContext ctx) {
    return Consumer<SegmentsProvider>(
      builder: (context, segmentsProvider, child) {
        developer.log(
          "[SegmentsConsumer] length=${segmentsProvider.segments().length}",
        );
        return Center(
          child: Container(
            constraints: const BoxConstraints(maxWidth: 1500),
            child: Column(
              children: [
                ElevatedButton(
                  onPressed: () => _selectTime(context),
                  child: Text(timeAsString()),
                ),
                Selector(
                  min: 8.0,
                  max: 30.0,
                  text: speedAsString(),
                  value: selectedSpeed * 3.6,
                  onChanged: (value) {
                    setState(() {
                      selectedSpeed = value * 1000 / 3600;
                    });
                  },
                ),
                Selector(
                  min: 50.0,
                  max: 150.0,
                  text: segmentLengthAsString(),
                  value: selectedSegmentLength / 1000,
                  onChanged: (value) {
                    setState(() {
                      selectedSegmentLength = value * 1000;
                    });
                  },
                ),
                ElevatedButton(
                  onPressed: () => onDone(context),
                  child: const Text("OK"),
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}

class SettingsWidget extends StatelessWidget {
  const SettingsWidget({super.key});

  Widget wait() {
    return Scaffold(
      appBar: AppBar(title: const Text('Segments')),
      body: Center(child: Column(children: [Text("loading...")])),
    );
  }

  @override
  Widget build(BuildContext ctx) {
    return Consumer<RootModel>(
      builder: (context, rootModel, child) {
        if (rootModel.provider() == null) {
          return wait();
        }
        developer.log(
          "[SegmentsProviderWidget] ${rootModel.provider()?.filename()} length=${rootModel.provider()?.segments().length}",
        );
        return ChangeNotifierProvider.value(
          value: rootModel.provider(),
          builder: (context, child) {
            return Scaffold(
              appBar: AppBar(title: const Text('Segments')),
              body: SegmentsSettings(),
            );
          },
        );
      },
    );
  }
}

class WidthSettings extends StatefulWidget {
  const WidthSettings({super.key});

  @override
  State<WidthSettings> createState() => _WidthSettingsState();
}

class _WidthSettingsState extends State<WidthSettings> {
  double width = 200;

  void onDone(BuildContext context) {
    SegmentsProvider provider = Provider.of<SegmentsProvider>(
      context,
      listen: false,
    );
    provider.setSmoothWidth(width);
  }

  String widthAsString() {
    return "$width m";
  }

  void onChanged(double value, SegmentsProvider provider) {
    provider.setSmoothWidth(value);
    setState(() {
      width = value;
    });
  }

  @override
  Widget build(BuildContext ctx) {
    return Consumer<SegmentsProvider>(
      builder: (context, segmentsProvider, child) {
        developer.log(
          "[_WidthSettingsState] length=${segmentsProvider.segments().length}",
        );
        return Center(
          child: Container(
            constraints: const BoxConstraints(maxWidth: 1500),
            child: Column(
              children: [
                Selector(
                  min: 10.0,
                  max: 1000.0,
                  text: widthAsString(),
                  value: width,
                  onChanged: (value) => onChanged(value,segmentsProvider),
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}
