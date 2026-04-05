import subprocess
import sys

steps = [
    "step1_compress.py",
    "step2_mockui.py",
    "step3_parse.py",
    "step4_dag.py",
    "step5_tasks.py",
    "step6_build.py",
]

for step in steps:
    print(f"\n{'='*40}")
    print(f"Running {step}")
    print('='*40)
    result = subprocess.run([sys.executable, step])
    if result.returncode != 0:
        print(f"ERROR: {step} failed. Stopping.")
        sys.exit(1)

print("\n✓ Pipeline complete. Check /output/")
