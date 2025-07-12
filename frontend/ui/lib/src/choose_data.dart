import 'dart:developer' as developer;
import 'package:file_picker/file_picker.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/routes.dart';

class ChooseData extends StatefulWidget {
  const ChooseData({super.key});

  @override
  State<ChooseData> createState() => _ChooseDataState();
}

class FindResult {
  List<int>? bytes;
  String? filename;
  bool demo = false;

  static FindResult makeFilenameResult(String filename) {
    var ret = FindResult();
    ret.filename = filename;
    return ret;
  }

  static FindResult makeBytesResult(List<int> bytes) {
    var ret = FindResult();
    ret.bytes = bytes;
    return ret;
  }

  static FindResult makeDemoResult() {
    var ret = FindResult();
    ret.demo = true;
    return ret;
  }
}

class _ChooseDataState extends State<ChooseData> {
  FindResult? findResult;

  void chooseGPX(RootModel rootModel) async {
    FilePickerResult? result = await FilePicker.platform.pickFiles(
      type: FileType.any,
    );
    if (result == null) {
      return;
    }
    developer.log("result: ${result.count}");
    FindResult? findResult;
    for (var file in result.files) {
      if (!kIsWeb) {
        findResult = FindResult.makeFilenameResult(file.path!);
      } else {
        var bytes = file.bytes!.buffer.asInt8List().toList();
        findResult = FindResult.makeBytesResult(bytes);
      }
      break;
    }
    if (mounted) {
      onDone(rootModel, findResult!);
    }
  }

  void onDone(RootModel model, FindResult findResult) {
    model.unload();
    if (findResult.demo) {
      model.createSegmentsProviderForDemo();
    } else if (findResult.bytes != null) {
      model.createSegmentsProviderFromBytes(findResult.bytes!);
    } else if (findResult.filename != null) {
      model.createSegmentsProvider(findResult.filename!);
    } else {
      assert(false);
    }
    developer.log("[push]");
    Navigator.of(context).pushNamed(RouteManager.settingsView);
  }

  void chooseDemo(RootModel model) {
    onDone(model, FindResult.makeDemoResult());
  }

  Widget buildFromModel(BuildContext ctx, RootModel rootModel, Widget? child) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          ElevatedButton(
            onPressed: () => chooseGPX(rootModel),
            child: const Text("Choose GPX file"),
          ),
          const SizedBox(height: 20),
          ElevatedButton(
            onPressed: () => chooseDemo(rootModel),
            child: const Text("Demo"),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext ctx) {
    return Consumer<RootModel>(
      builder: (context, rootModel, child) {
        return buildFromModel(context, rootModel, child);
      },
    );
  }
}

class HomePage extends StatelessWidget {
  const HomePage({super.key});

  @override
  Widget build(BuildContext ctx) {
    return ChooseData();
  }
}
