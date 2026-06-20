import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/widgets/waypoints_table_widget.dart';

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
            child: GPXTable(kinds: [Kind.controls]),
          ),
        ),
        Divider(),
        SizedBox(height: 30),
      ],
    );

    return Scaffold(
      appBar: AppBar(title: const Text('Control Points Table')),
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
  final SegmentModel model;
  const ControlsTableScreen({super.key, required this.model});

  @override
  Widget build(BuildContext context) {
    return ChangeNotifierProvider.value(
      value: model,
      builder: (innercontext, child) {
        return ControlsTableWidget();
      },
    );
  }
}
