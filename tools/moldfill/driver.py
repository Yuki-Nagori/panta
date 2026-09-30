"""将 Panta 冻结的运行参数转换为 Moldfill YAML，再调用外部引擎 CLI。

只做格式与路径适配；网格、浇口推荐和充填算法仍由外部求解器执行。
每次保留独立 case.yaml，便于复现 GUI 实际使用的参数。
"""

import json
import pathlib
import sys

import yaml


def main():
    """读取单次请求，固化绝对资产路径并执行指定求解阶段。"""
    request = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
    template = pathlib.Path(request["template"])
    case = yaml.safe_load(template.read_text(encoding="utf-8"))
    case["material"] = str((template.parent / case["material"]).resolve())
    case["geometry"]["file"] = request["source"]
    case["geometry"]["units"] = "mm"
    case["geometry"]["remesh"]["edge_length"] = request["edgeLength"]
    process = case["process"]
    process["mold_temp_c"] = request["moldTemperature"]
    process["melt_temp_c"] = request["meltTemperature"]
    process["flow_rate_cm3_s"] = request["flowRate"]
    process["vp_switch"] = {"by_volume": request["switchVolume"] / 100.0}
    case["gate"] = {"mode": "auto", "top_k": 3}
    if request["mode"] == "solve":
        recommendation = yaml.safe_load(pathlib.Path(request["gate"]).read_text(encoding="utf-8"))
        if recommendation["schema"] != "moldfill.gate_recommend/v1":
            raise ValueError("Unsupported gate recommendation")
        # 节点编号只在同一重划网格上有效；宿主固定 mesh 与推荐产物。
        case["gate"] = {"mode": "nodes", "nodes": recommendation["resolved_gate_nodes"]}
        print("Using resolved gate nodes:", case["gate"]["nodes"], flush=True)
        print("Fill only: holding profile, fiber orientation and crystallization are not solved.", flush=True)
    destination = pathlib.Path(request["output"])
    case_path = destination / "case.yaml"
    case_path.write_text(yaml.safe_dump(case, sort_keys=False), encoding="utf-8")
    argv = ["run", str(case_path), "--out", str(destination)]
    if request["mode"] == "remesh_only":
        argv += ["--remesh-only"]
    else:
        argv += ["--mesh", request["mesh"]]
        if request["mode"] == "gate_only":
            argv += ["--gate-only"]
    from moldfill.cli import main as solver_main
    status = solver_main(argv)
    if status == 0 and request["mode"] == "gate_only":
        runs = [p for p in (destination / "runs").iterdir() if not p.name.endswith(".staging")]
        if len(runs) != 1:
            raise ValueError("Expected one committed gate run")
        recommendation = yaml.safe_load((runs[0] / "artifacts/gate_recommend.yaml").read_text(encoding="utf-8"))
        selected = recommendation["gate"]["recommended"][0]["position_m"]
        (destination / "gate-display.json").write_text(json.dumps([v * 1000 for v in selected]), encoding="utf-8")
    return status


if __name__ == "__main__":
    sys.exit(main())
