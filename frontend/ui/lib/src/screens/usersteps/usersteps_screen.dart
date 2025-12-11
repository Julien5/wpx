import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/wheel/wheel_screen.dart';
import 'package:ui/src/widgets/userstepsslider.dart';

class UserStepsTable extends StatelessWidget {
  const UserStepsTable({super.key});

  String _formatDistance(double distance) {
    final km = distance / 1000.0;
    return NumberFormat('0.0').format(km);
  }

  String _formatSlope(double slope) {
    final percent = slope * 100;
    final n = NumberFormat('#0.0').format(percent);
    return "$n%";
  }

  String _formatTime(String rfc3339Time) {
    final dateTime = DateTime.parse(rfc3339Time).toLocal();
    return DateFormat('HH:mm').format(dateTime);
  }

  Widget buildData(List<Waypoint> waypoints) {
    if (waypoints.isEmpty) {
      return Center(child: const Text("No waypoints"));
    }
    return DataTable(
      // 1. Define the Columns
      columns: const <DataColumn>[
        DataColumn(
          label: Text('km', style: TextStyle(fontWeight: FontWeight.bold)),
          numeric: true,
        ),
        DataColumn(
          label: Text('Time', style: TextStyle(fontWeight: FontWeight.bold)),
          numeric: true,
        ),
        DataColumn(
          label: Text('Slope', style: TextStyle(fontWeight: FontWeight.bold)),
          numeric: true,
        ),
      ],
      // 2. Map Waypoints to Data Rows
      rows:
          waypoints.map((w) {
            final formattedDistance = _formatDistance(w.info!.distance);
            final formattedTime = _formatTime(w.info!.time);
            final formattedSlope = _formatSlope(w.info!.interSlope);

            return DataRow(
              cells: <DataCell>[
                // Distance Cell
                DataCell(Text(formattedDistance)),
                // Time Cell
                DataCell(Text(formattedTime)),
                DataCell(Text(formattedSlope)),
              ],
            );
          }).toList(),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Consumer<SegmentModel>(
      builder: (context, model, child) {
        Set<InputType> usersteps = {InputType.userStep};
        var waypoints = model.someWaypoints(usersteps);
        return SingleChildScrollView(
          scrollDirection: Axis.vertical,
          child: buildData(waypoints),
        );
      },
    );
  }
}

class UserStepsScreen extends StatelessWidget {
  const UserStepsScreen({super.key});

  Widget wait() {
    return Scaffold(
      appBar: AppBar(title: const Text('Segments')),
      body: Center(child: Column(children: [Text("loading...")])),
    );
  }

  @override
  Widget build(BuildContext ctx) {
    Set<InputType> usersteps = {InputType.userStep};
    return Scaffold(
      appBar: AppBar(title: const Text('Pacing Points')),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.start,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            WheelWidget(kinds: usersteps),
            Divider(),
            SizedBox(height: 10),
            UserStepsSliderProvider(),
            SizedBox(height: 10),
            Divider(),
            Expanded(
              child: Card(
                elevation: 4, // Add shadow to the card
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(8), // Rounded corners
                ),
                child: UserStepsTable(),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class UserStepsProvider extends StatelessWidget {
  const UserStepsProvider({super.key});

  @override
  Widget build(BuildContext context) {
    RootModel root = Provider.of<RootModel>(context);
    Bridge bridge = root.getBridge();
    assert(bridge.isLoaded());
    return ChangeNotifierProvider(
      create: (ctx) => SegmentModel(bridge, root.trackSegment()),
      builder: (context, child) {
        return UserStepsScreen();
      },
    );
  }
}
