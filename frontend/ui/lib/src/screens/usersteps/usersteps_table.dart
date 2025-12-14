import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';

class TableWidget extends StatelessWidget {
  const TableWidget({super.key});

  String _formatDistance(double distance) {
    final km = distance / 1000.0;
    return NumberFormat('0.0').format(km);
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
          label: Text('Name', style: TextStyle(fontWeight: FontWeight.bold)),
          numeric: true,
        ),
      ],
      // 2. Map Waypoints to Data Rows
      rows:
          waypoints.map((w) {
            final formattedDistance = _formatDistance(w.info!.distance);
            final gpxName = w.info!.gpxName;

            return DataRow(
              cells: <DataCell>[
                // Distance Cell
                DataCell(Text(formattedDistance)),
                // Time Cell
                DataCell(Text(gpxName)),
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

class UserStepsTableWidget extends StatelessWidget {
  const UserStepsTableWidget({super.key});

  @override
  Widget build(BuildContext ctx) {
    return Scaffold(
      appBar: AppBar(title: const Text('Pacing Points Table')),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.start,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            Divider(),
            SizedBox(height: 30),
            Expanded(
              child: Card(
                elevation: 4, // Add shadow to the card
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(8), // Rounded corners
                ),
                child: TableWidget(),
              ),
            ),
            Divider(),
            SizedBox(height: 30),
          ],
        ),
      ),
    );
  }
}

class SegmentModelReceiver extends StatelessWidget {
  const SegmentModelReceiver({super.key});

  @override
  Widget build(BuildContext context) {
    assert(ModalRoute.of(context) != null);
    assert(ModalRoute.of(context)!.settings.arguments != null);
    var arg = ModalRoute.of(context)!.settings.arguments;
    SegmentModel model = arg as SegmentModel;
    return ChangeNotifierProvider.value(
      value: model,
      builder: (innercontext, child) {
        return UserStepsTableWidget();
      },
    );
  }
}

typedef UserStepsTableScreen = SegmentModelReceiver;
