Import("env")
import os

env_file = os.path.join(env.get("PROJECT_DIR"), ".env")
if os.path.exists(env_file):
    print(f"[load_env.py] Loading environment variables from {env_file}")
    with open(env_file) as f:
        for line in f:
            line = line.strip()
            if line and not line.startswith("#") and "=" in line:
                key, val = line.split("=", 1)
                key = key.strip()
                val = val.strip().strip("\"'")
                # Add as C preprocessor definition: -D KEY="value"
                env.Append(CPPDEFINES=[(key, env.StringifyMacro(val))])
                print(f"[load_env.py] Defined {key}")
