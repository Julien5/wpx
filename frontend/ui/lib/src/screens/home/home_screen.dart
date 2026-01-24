import 'dart:developer' as developer;
import 'dart:io';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/screens/load/load_screen.dart';

class ChooseData extends StatefulWidget {
  const ChooseData({super.key});

  @override
  State<ChooseData> createState() => _ChooseDataState();
}

class UserInput {
  List<int>? bytes;
  String? filename;
  bool demo = false;

  static UserInput makeFromBytes(List<int> bytes) {
    var ret = UserInput();
    ret.bytes = bytes;
    return ret;
  }

  static UserInput makeDemo() {
    var ret = UserInput();
    ret.demo = true;
    return ret;
  }
}

class _ChooseDataState extends State<ChooseData> {
  UserInput? findResult;
  String? errorMessage;
  bool loading = false;

  void chooseGPX() async {
    FilePickerResult? result = await FilePicker.platform.pickFiles(
      type: FileType.custom,
      allowedExtensions: ["gpx"],
    );
    if (result == null) {
      return;
    }
    if (!mounted) {
      return;
    }
    developer.log("result: ${result.count}");
    for (var file in result.files) {
      List<int> bytes = [];
      if (file.bytes == null) {
        bytes = await File(file.path!).readAsBytes();
      } else {
        bytes = file.bytes!.buffer.asInt8List().toList();
      }
      onDone(UserInput.makeFromBytes(bytes));
      break;
    }
  }

  void chooseDemo() {
    onDone(UserInput.makeDemo());
  }

  void gotoLoad(BuildContext ctx, UserInput userInput) {
    Navigator.push(
      ctx,
      MaterialPageRoute(
        builder: (context) => LoadProvider(userInput: userInput),
      ),
    );
  }

  void onDone(UserInput userInput) async {
    gotoLoad(context, userInput);
  }

  Widget buildFromModel(BuildContext ctx, RootModel rootModel, Widget? child) {
    return Center(
      child: Column(
        children: [
          SizedBox(height: 40),
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Image.asset(
                'assets/images/png/combined.png',
                width: 250,
                fit: BoxFit.cover,
              ),
            ],
          ),
          SizedBox(height: 40),
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              ElevatedButton(
                onPressed: loading ? null : () => chooseGPX(),
                child: const Text("GPX file"),
              ),
              if (errorMessage !=
                  null) // Conditionally display the error message
                Padding(
                  padding: const EdgeInsets.only(top: 10),
                  child: Text(
                    errorMessage!,
                    style: const TextStyle(color: Colors.red),
                  ),
                ),
              const SizedBox(width: 20),
              ElevatedButton(
                onPressed: loading ? null : () => chooseDemo(),
                child: const Text("Demo"),
              ),
            ],
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

class HomeScreen extends StatelessWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext ctx) {
    return ChooseData();
  }
}
