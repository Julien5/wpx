import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/screens/desktop/kinds_row.dart';
import 'package:wpx/src/screens/settings/settings_screen.dart';
import 'package:wpx/src/screens/wheel/statistics_widget.dart';
import 'package:wpx/src/utils/print.dart';
import 'package:wpx/src/widgets/export.dart';
import 'package:wpx/src/widgets/small.dart';
import 'package:wpx/src/widgets/userstepsslider.dart';
import 'package:wpx/src/utils/utils.dart';

class UserStepsCard extends StatelessWidget {
  final ExpansibleController controller;
  const UserStepsCard({super.key, required this.controller});

  @override
  Widget build(BuildContext context) {
    SegmentModel parameterModel = context.watch<SegmentModel>();

    String pacingPointsText = getPacingPointText(parameterModel.parameters());
    Widget tile = ExpansionTile(
      title: Row(
        children: [
          SmallText(text: "Cutoff points"),
          SmallText(text: pacingPointsText),
        ],
      ),
      controller: controller,
      children: <Widget>[
        Builder(
          builder: (BuildContext context) {
            return UserStepsSliderProvider();
          },
        ),
      ],
    );
    return Card(elevation: 4, child: tile);
  }
}

class PDFCard extends StatelessWidget {
  final ExpansibleController controller;
  const PDFCard({super.key, required this.controller});

  @override
  Widget build(BuildContext context) {
    SegmentModel segment = context.watch<SegmentModel>();
    List<Segment> segments = segment.backend.segments();
    String segLength = (segmentLengthWithoutOverlap(segment.parameters()) /
            1000)
        .ceil()
        .toString()
        .padLeft(3);
    String pageCount = PageCountInfo.getPagesCountString(segments.length);
    Widget tile = ExpansionTile(
      title: Row(
        children: [
          SmallText(text: "PDF"),
          SmallText(text: "$pageCount, $segLength km per page"),
        ],
      ),
      controller: controller,
      children: <Widget>[
        Row(
          children: [SmallText(text: "Number of pages:"), PagesSliderWidget()],
        ),
      ],
    );
    return Card(elevation: 4, child: tile);
  }
}

class SidePanel extends StatefulWidget {
  final double width;
  final String? activeMode;
  final void Function(String? mode) onModeChanged;
  const SidePanel({
    super.key,
    required this.width,
    required this.activeMode,
    required this.onModeChanged,
  });

  @override
  State<SidePanel> createState() => _SidePanelState();
}

class _SidePanelState extends State<SidePanel> {
  final ExpansibleController _userStepsController = ExpansibleController();
  final ExpansibleController _pdfController = ExpansibleController();

  // Guard: when _syncControllersFromMode programmatically changes controllers,
  // the listeners fire during build (didUpdateWidget). We ignore those calls.
  bool _syncing = false;

  @override
  void initState() {
    super.initState();
    _userStepsController.addListener(_onUserStepsChanged);
    _pdfController.addListener(_onPdfChanged);
    _syncing = true;
    _syncControllersFromMode();
    _syncing = false;
  }

  @override
  void dispose() {
    _userStepsController.removeListener(_onUserStepsChanged);
    _pdfController.removeListener(_onPdfChanged);
    super.dispose();
  }

  void _onUserStepsChanged() {
    if (_syncing) return;
    widget.onModeChanged(_userStepsController.isExpanded ? 'usersteps' : null);
  }

  void _onPdfChanged() {
    if (_syncing) return;
    widget.onModeChanged(_pdfController.isExpanded ? 'settings' : null);
  }

  @override
  void didUpdateWidget(SidePanel old) {
    super.didUpdateWidget(old);
    if (widget.activeMode != old.activeMode) {
      _syncing = true;
      _syncControllersFromMode();
      _syncing = false;
    }
  }

  void _syncControllersFromMode() {
    if (widget.activeMode == 'usersteps') {
      _userStepsController.expand();
      _pdfController.collapse();
    } else if (widget.activeMode == 'settings') {
      _userStepsController.collapse();
      _pdfController.expand();
    } else {
      _userStepsController.collapse();
      _pdfController.collapse();
    }
  }

  @override
  Widget build(BuildContext context) {
    Widget div = Divider(color: Colors.lightBlue, thickness: 1, height: 1);
    List<Widget> children = [
      div,
      Padding(
        padding: const EdgeInsets.all(15),
        child: Column(
          children: [
            OverviewWidget(
              onPacingPointPressed: null,
              onControlsPointPressed: null,
              onPDFPressed: null,
            ),
            Card(elevation: 4, child: KindsRow()),
          ],
        ),
      ),
      div,
      Padding(
        padding: const EdgeInsets.all(15),
        child: UserStepsCard(controller: _userStepsController),
      ),
      div,
      Padding(
        padding: const EdgeInsets.all(15),
        child: PDFCard(controller: _pdfController),
      ),
      div,
      SizedBox(height: 20),
      Center(child: ExportButton(text: "export zip")),
    ];
    return ConstrainedBox(
      constraints: BoxConstraints(
        minWidth: widget.width,
        maxWidth: widget.width,
      ),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.start,
        children: children,
      ),
    );
  }
}
