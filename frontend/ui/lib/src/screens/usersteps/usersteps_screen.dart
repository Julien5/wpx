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

  // Helper function to format the distance
  String _formatDistance(double distance) {
    // Assuming the 'distance' field in WaypointInfo is in meters.
    // Convert meters to kilometers and round to the nearest integer.
    final km = distance / 1000.0;
    return NumberFormat('0').format(km); // '0' ensures no decimal precision
  }

  String _formatSlope(double slope) {
    // Assuming the 'distance' field in WaypointInfo is in meters.
    // Convert meters to kilometers and round to the nearest integer.
    final percent = slope * 100;
    final n = NumberFormat(
      '#0.0',
    ).format(percent); // '0' ensures no decimal precision
    return "$n%";
  }

  // Helper function to format the time
  String _formatTime(String rfc3339Time) {
    try {
      final dateTime = DateTime.parse(rfc3339Time).toLocal();
      // 'HH' is 24-hour format, 'mm' is minute.
      return DateFormat('HH:mm').format(dateTime);
    } catch (e) {
      // Handle parsing errors gracefully
      return 'N/A';
    }
  }

  Widget buildTable(BuildContext context, List<Waypoint> waypoints) {
    return SingleChildScrollView(
      scrollDirection: Axis.vertical,
      child: DataTable(
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
      ),
    );
  }

  Widget embed(BuildContext context, Widget table) {
    return LayoutBuilder(
      builder: (context, constraints) {
        return ConstrainedBox(
          constraints: const BoxConstraints(maxHeight: 300),
          child: table,
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    return Consumer<SegmentModel>(
      builder: (context, model, child) {
        Set<InputType> usersteps = {InputType.userStep};
        var waypoints = model.someWaypoints(usersteps);
        if (waypoints.isEmpty) {
          return const Text("No waypoints");
        }
        //return Text("${waypoints.length} waypoints");
        return embed(context, buildTable(context, waypoints));
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

  void goback(BuildContext ctx) {
    Navigator.of(ctx).pop();
  }

  @override
  Widget build(BuildContext ctx) {
    Set<InputType> usersteps = {InputType.userStep};
    return Scaffold(
      appBar: AppBar(title: const Text('Pacing Points')),
      body: Center(
        child: Container(
          constraints: const BoxConstraints(maxWidth: 500),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.start,
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              WheelWidget(kinds: usersteps),
              SizedBox(height: 10),
              UserStepsSliderProvider(),
              SizedBox(height: 10),
              UserStepsTable(),
            ],
          ),
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
