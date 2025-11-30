import 'dart:developer' as developer;
import 'dart:math' as math;
import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/src/widgets/slidervalues.dart';
import 'package:ui/utils.dart';

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
    developer.log("[selector/build] text=$text value=$value");
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

class WheelScreen extends StatefulWidget {
  const WheelScreen({super.key});

  @override
  State<WheelScreen> createState() => _WheelScreenState();
}

List<double> segmentLengthSliderValues(double trackLength) {
  double trackLengthKm = trackLength / 1000;
  List<double> values = [2, 5, 10];
  if (trackLengthKm > 10) {
    values = [5, 10, 25, 50];
  }
  if (trackLengthKm > 50) {
    values = [10, 25, 50, 100];
  }
  if (trackLengthKm > 100) {
    values = [25, 50, 100, 150, 200];
  }
  if (trackLengthKm > 200) {
    values = [50, 100, 150, 200, 400];
  }
  if (trackLengthKm > 400) {
    values = [100, 150, 200, 300, 600];
  }
  if (trackLengthKm > 600) {
    values = [100, 150, 200, 300, 600, 1000];
  }
  return fromKm(values);
}

List<double> fromKmh(List<double> list) {
  List<double> ret = list;
  for (int k = 0; k < list.length; ++k) {
    ret[k] = list[k] * 1000/3600;
  }
  return ret;
}

List<double> speedSliderValues() {
  return fromKmh([5, 10, 12.5, 13.5,15, 18.0, 20, 25, 28]);
}

class _WheelScreenState extends State<WheelScreen> {
  DateTime startTime = DateTime.now();
  final SliderValues _segmentLengthSliderValues = SliderValues();
  final SliderValues _speedSliderValues = SliderValues();

  @override
  void initState() {
    super.initState();

    WidgetsBinding.instance.addPostFrameCallback((_) {
      //? readModel();
      RootModel rootModel = Provider.of<RootModel>(context, listen: false);
      developer.log("E=${rootModel.statistics().distanceEnd}");

      double trackLength = rootModel.statistics().distanceEnd;
      var values = segmentLengthSliderValues(trackLength);
      _segmentLengthSliderValues.init(values, trackLength / 2);

      values = speedSliderValues();
      _speedSliderValues.init(values, 15/3.6);
      setState(() {});
    });
  }

  void readModel() {
    RootModel rootModel = Provider.of<RootModel>(context, listen: false);
    bridge.Parameters parameters = rootModel.parameters();
    startTime = DateTime.parse(parameters.startTime);
    _speedSliderValues.setValue(parameters.speed);
    _segmentLengthSliderValues.setValue(parameters.segmentLength/1.1);
  }

  void writeModel(BuildContext context) {
    RootModel rootModel = Provider.of<RootModel>(context, listen: false);
    bridge.Parameters oldParameters = rootModel.parameters();
    String rfc3339time = startTime.toIso8601String();
    if (!rfc3339time.endsWith("Z")) {
      rfc3339time = "${rfc3339time}Z";
    }
    var realLength = _segmentLengthSliderValues.current()*1.1;
    var overlap = _segmentLengthSliderValues.current()*0.1;
    bridge.Parameters newParameters = bridge.Parameters(
      speed: _speedSliderValues.current(),
      startTime: rfc3339time,
      segmentLength: realLength,
      segmentOverlap: overlap,
      smoothWidth: oldParameters.smoothWidth,
      profileOptions: oldParameters.profileOptions,
      mapOptions: oldParameters.mapOptions,
      debug: oldParameters.debug,
    );
    rootModel.setParameters(newParameters);
    Navigator.of(context).pushNamed(RouteManager.segmentsView);
  }

  Future<void> _selectTime(BuildContext context) async {
    final TimeOfDay? picked = await showTimePicker(
      context: context,
      initialTime: TimeOfDay(hour: startTime.hour, minute: startTime.minute),
    );

    if (picked != null) {
      DateTime dateTime = DateTime(
        startTime.year,
        startTime.month,
        startTime.day,
        picked.hour,
        picked.minute,
      );
      setState(() {
        startTime = dateTime;
      });
    }
  }

  Future<void> _selectDate(BuildContext context) async {
    final DateTime? picked = await showDatePicker(
      context: context,
      firstDate: DateTime.now(),
      lastDate: DateTime.now().add(Duration(days: 30)),
      initialDate: startTime,
    );

    if (picked != null) {
      DateTime pickedDateTime = DateTime(
        picked.year,
        picked.month,
        picked.day,
        startTime.hour,
        startTime.minute,
      );
      setState(() {
        startTime = pickedDateTime;
      });
    }
  }

  String timeAsString() {
    return DateFormat('HH:mm').format(startTime);
  }

  String dateAsString() {
    return DateFormat('dd.MM.yyyy').format(startTime);
  }

  String speedAsString() {
    double kmh = _speedSliderValues.current() * 3.6;
    return "Speed: ${kmh.toStringAsFixed(1)} kmh";
  }

  String segmentLengthAsString() {
    double km = _segmentLengthSliderValues.current() / 1000;
    return "Segment length: ${km.toStringAsFixed(1)} km";
  }

  @override
  Widget build(BuildContext ctx) {
    Table table1 = Table(
      columnWidths: const {
        0: FlexColumnWidth(), // Fixed width for the first column
        1: FlexColumnWidth(), // Flexible width for the second column
      },
      children: [
        TableRow(
          children: [
            Container(
              alignment: Alignment.center,
              child: const Text("Start Date:"),
            ),
            Container(
              alignment: Alignment.center,
              child: ElevatedButton(
                onPressed: () => _selectDate(context),
                child: Text(dateAsString()),
              ),
            ),
          ],
        ),
        TableRow(
          children: [
            Container(
              height: 50,
              alignment: Alignment.center,
              child: const Text("Start Time:"),
            ),
            Container(
              height: 50,
              alignment: Alignment.center,
              child: ElevatedButton(
                onPressed: () => _selectTime(context),
                child: Text(timeAsString()),
              ),
            ),
          ],
        ),
      ],
    );
    Card card1 = Card(
      elevation: 4, // Add shadow to the card
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(8), // Rounded corners
      ),
      child: Padding(
        padding: const EdgeInsets.all(16), // Add padding inside the card
        child: table1,
      ),
    );
    Column table2 = Column(
      children: [
        Row(
          children: [
            Expanded(
              child: Text(speedAsString()),
            ),
            Expanded(
              child: SliderValuesWidget(
                values: _speedSliderValues,
                onChanged:
                    (value) => setState(() {
                      _speedSliderValues.setValue(value);
                    }),
                formatLabel: (value) => "${(value*3600/1000).toStringAsFixed(1)} km/h",
              ),
            ),
          ],
        ),
        Row(
          children: [
            Expanded(
              child: Text(segmentLengthAsString()),
            ),
            Expanded(
              child: SliderValuesWidget(
                values: _segmentLengthSliderValues,
                onChanged:
                    (value) => setState(() {
                      _segmentLengthSliderValues.setValue(value);
                    }),
                formatLabel: (value) => "${(value / 1000).floor()} km",
              ),
            ),
          ],
        ),
      ],
    );
    Card card2 = Card(
      elevation: 4, // Add shadow to the card
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(8), // Rounded corners
      ),
      child: Padding(
        padding: const EdgeInsets.all(16), // Add padding inside the card
        child: table2,
      ),
    );
    return Column(
      children: [
        card1,
        card2,
        SizedBox(height: 10),
        ElevatedButton(
          onPressed: () => writeModel(context),
          child: const Text("Preview"),
        ),
      ],
    );
  }
}

class SettingsScreen extends StatelessWidget {
  const SettingsScreen({super.key});

  Widget wait() {
    return Scaffold(
      appBar: AppBar(title: const Text('Segments')),
      body: Center(child: Column(children: [Text("loading...")])),
    );
  }

  @override
  Widget build(BuildContext ctx) {
    Card infoCard = Card(
      elevation: 4, // Add shadow to the card
      margin: const EdgeInsets.all(1), // Add margin around the card
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(8), // Rounded corners
      ),
      child: Padding(
        padding: const EdgeInsets.all(16), // Add padding inside the card
        child: Text("HI"),
      ),
    );

    return Scaffold(
      appBar: AppBar(title: const Text('Wheel')),
      body: Center(
        child: Container(
          constraints: const BoxConstraints(maxWidth: 500),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [infoCard, SizedBox(height: 15), WheelScreen()],
          ),
        ),
      ),
    );
  }
}
