import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/waypoints_table_widget.dart';

class ControlsTableWidget extends StatelessWidget {
  const ControlsTableWidget({super.key});

  @override
  Widget build(BuildContext ctx) {
    Widget column = Column(
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
            child: WaypointsTableWidget(kind: InputType.control),
          ),
        ),
        Divider(),
        SizedBox(height: 30),
      ],
    );

    return Scaffold(
      appBar: AppBar(title: const Text('Pacing Points Table')),
      body: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(
            maxWidth: 400,
          ), // Set max width to 400px
          child: column,
        ),
      ),
    );
  }
}

class ControlsTableScreen extends StatelessWidget {
  const ControlsTableScreen({super.key});

  @override
  Widget build(BuildContext context) {
    assert(ModalRoute.of(context) != null);
    assert(ModalRoute.of(context)!.settings.arguments != null);
    var arg = ModalRoute.of(context)!.settings.arguments;
    SegmentModel model = arg as SegmentModel;
    return ChangeNotifierProvider.value(
      value: model,
      builder: (innercontext, child) {
        return ControlsTableWidget();
      },
    );
  }
}
