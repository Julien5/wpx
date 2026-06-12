import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/models/stackviewscontroller.dart';
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/widgets/simpletrackview.dart';

class _SegmentGraphicsButtons extends StatelessWidget {
  final VoidCallback? onPressed;
  final RenderFunction trackData;
  final RenderFunction selected;
  final double size;
  const _SegmentGraphicsButtons({
    required this.selected,
    required this.size,
    required this.trackData,
    this.onPressed,
  });
  final double margin = 8;
  Image icon(RenderFunction data) {
    String filename = 'assets/icons/png/map.png';
    if (data == RenderFunction.wheel) {
      filename = 'assets/icons/png/clock.png';
    } else if (data == RenderFunction.profile) {
      filename = 'assets/icons/png/profile.png';
    } else if (data == RenderFunction.map) {
      filename = 'assets/icons/png/map.png';
    } else if (data == RenderFunction.wheelPages) {
      filename = 'assets/icons/png/clock.png';
    } else {
      assert(false, "no icon for $data");
    }
    return Image.asset(filename, width: size - margin, height: size - margin);
  }

  @override
  Widget build(BuildContext context) {
    double frameWidth = 1.0;
    if (selected == trackData) {
      frameWidth = 3.0;
    }
    return Container(
      width: size,
      height: size,
      decoration: BoxDecoration(
        color: Colors.white,
        border: Border.all(color: Colors.black, width: frameWidth),
        borderRadius: BorderRadius.circular(margin),
      ),
      child: IconButton(
        padding: EdgeInsets.zero,
        constraints: BoxConstraints.tight(Size(size, size)),
        icon: icon(trackData),
        onPressed: onPressed,
      ),
    );
  }
}

class SegmentGraphicsButtonsColumn extends StatelessWidget {
  final void Function(RenderFunction) onButtonPressed;
  final RenderFunction selected;
  final double size;

  const SegmentGraphicsButtonsColumn({
    super.key,
    required this.selected,
    required this.size,
    required this.onButtonPressed,
  });

  @override
  Widget build(BuildContext context) {
    StackViewsController model = Provider.of<StackViewsController>(context);
    if (model.exposed.length <= 1) {
      return SizedBox();
    }
    const double buttonSize = 30;
    List<Widget> children = [];
    for (RenderFunction data in model.exposed) {
      children.add(
        Padding(
          padding: const EdgeInsetsGeometry.fromLTRB(
            0,
            buttonSize / 10,
            0,
            buttonSize / 10,
          ),
          child: _SegmentGraphicsButtons(
            selected: selected,
            size: buttonSize,
            trackData: data,
            onPressed: () => onButtonPressed(data),
          ),
        ),
      );
    }
    return ConstrainedBox(
      constraints: const BoxConstraints(maxWidth: buttonSize),
      child: Column(
        mainAxisSize: MainAxisSize.max,
        mainAxisAlignment: MainAxisAlignment.spaceEvenly,
        children: children,
      ),
    );
  }
}

class SegmentGraphics extends StatefulWidget {
  final Kinds kinds;
  const SegmentGraphics({super.key, required this.kinds});

  @override
  State<SegmentGraphics> createState() => _SegmentGraphicsState();
}

class _SegmentGraphicsState extends State<SegmentGraphics> {
  Map<RenderFunction, Widget> widgets = {};

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (widgets.isNotEmpty) {
      return;
    }

    StackViewsController model = Provider.of<StackViewsController>(
      context,
      listen: false,
    );

    for (RenderFunction data in model.exposed) {
      widgets[data] = SimpleTrackView.make(widget.kinds, data);
    }
    setState(() {});
  }

  void onTap() {
    StackViewsController model = Provider.of<StackViewsController>(
      context,
      listen: false,
    );
    model.cycle();
  }

  @override
  Widget build(BuildContext ctx) {
    // Instanciating a Provider.of<Model>(context) (listen=true)
    // is necessary to get rebuild on notifyListeners.
    StackViewsController model = Provider.of<StackViewsController>(context);
    double margin = 8;
    RenderFunction currentModelData = model.currentData();
    // I would like to have `visible = widgets[currentModelData]`
    // but then the widget states are disposed.
    // AI says: In Flutter, when you swap a widget out of the build tree,
    // the previous widget is unmounted and its State object is disposed of.
    // Solution: Stack with Offstaged widgets.
    Widget visible = Stack(
      fit: StackFit.expand, // <--- Add this line
      children:
          widgets.entries.map((entry) {
            return Offstage(
              offstage: entry.key != currentModelData,
              child: entry.value,
            );
          }).toList(),
    );
    return GestureDetector(
      onTap: onTap,
      child: Padding(
        padding: EdgeInsetsGeometry.fromLTRB(margin, margin, margin, margin),
        child: visible,
      ),
    );
  }
}

class TrackGraphicsRow extends StatelessWidget {
  final Kinds kinds;

  const TrackGraphicsRow({super.key, required this.kinds});

  void onButtonPressed(BuildContext context, RenderFunction data) {
    StackViewsController model = Provider.of<StackViewsController>(
      context,
      listen: false,
    );
    model.changeCurrent(data);
  }

  @override
  Widget build(BuildContext context) {
    // Instanciating a Provider.of<Model>(context) (listen=true)
    // is necessary to get rebuild on notifyListeners.
    StackViewsController model = Provider.of<StackViewsController>(context);
    RenderFunction currentModelData = model.currentData();
    developer.log("[TrackGraphicsRow] build currentData:$currentModelData");
    Widget buttonColumn = SegmentGraphicsButtonsColumn(
      onButtonPressed: (trackData) => {onButtonPressed(context, trackData)},
      selected: currentModelData,
      size: 30,
    );
    return Padding(
      padding: EdgeInsetsGeometry.fromLTRB(0, 0, 5, 0),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Expanded(child: SegmentGraphics(kinds: kinds)),
          buttonColumn,
        ],
      ),
    );
  }
}
