#!/bin/bash

# --- Configuration ---
# The script to be executed in parallel.
# Assumes it's in the same directory or in the system's PATH.
TARGET_SCRIPT="./run-proofs.sh"

# --- FUNCTION DATA ---
ALL_NAMES=$(sed -n -e 's/^| \(test_p[a-zA-Z0-9:_]*\) *|.*/\1/p' proofs.md)
MULTISIG_NAMES=$(sed -n -e 's/^| m | \(test_p[a-zA-Z0-9:_]*\) *|.*/\1/p' proofs.md)

echo $ALL_NAMES

# --- HARDCODED DATA ---
# The space-separated list of all function names to be processed.
# Modify this string to change the list of functions.
# FUNCTION_LIST_STRING="process_data validate_input generate_report cleanup_logs send_email update_database check_status"


# --- Argument Parsing and Validation ---

# Check for the correct number of arguments
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <functions_per_run>"
    echo "Note that lower functions per run make for more parallel runs."
    echo "Example: $0 3"
    exit 1
fi

# Get the argument from the command line
FUNCTIONS_PER_RUN=$1

# Validate that FUNCTIONS_PER_RUN is a positive integer
if ! [[ "$FUNCTIONS_PER_RUN" =~ ^[1-9][0-9]*$ ]]; then
    echo "Error: <functions_per_run> must be a positive integer."
    exit 1
fi

# Check if the target script exists and is executable
if ! command -v "$TARGET_SCRIPT" &> /dev/null; then
    echo "Error: Target script '$TARGET_SCRIPT' not found or not executable."
    echo "Please ensure it is in your PATH or the same directory."
    exit 1
fi


# --- Main Logic ---

# Convert the space-separated string of functions into a bash array
# read -ra ALL_FUNCTIONS <<< "$ALL_NAMES" # This is not working for some reason
#This is a hacky approach to get things going, but shouldn't be in production
ALL_FUNCTIONS=($ALL_NAMES)


# Get the total number of functions
TOTAL_FUNCTIONS=${#ALL_FUNCTIONS[@]}

echo
echo "Total functions to process: $TOTAL_FUNCTIONS"
echo "Functions per parallel run: $FUNCTIONS_PER_RUN"
echo "------------------------------------------------"
echo

# Calculate the number of parallel runs needed
# This uses bash arithmetic to round up: (numerator + denominator - 1) / denominator
PARALLEL_RUNS=$(( (TOTAL_FUNCTIONS + FUNCTIONS_PER_RUN - 1) / FUNCTIONS_PER_RUN ))

echo "Starting $PARALLEL_RUNS parallel job(s)..."
echo

# Loop from 0 to the number of runs - 1
for (( i=0; i<PARALLEL_RUNS; i++ )); do
    # Calculate the starting and ending index for the slice of the array
    START_INDEX=$(( i * FUNCTIONS_PER_RUN ))
    END_INDEX=$(( (i + 1) * FUNCTIONS_PER_RUN - 1 ))

    # Slice the array to get the functions for this specific run
    # This slice syntax works in bash 4.0+
    CURRENT_FUNCTIONS_SLICE=("${ALL_FUNCTIONS[@]:$START_INDEX:$FUNCTIONS_PER_RUN}")

    # Join the array elements back into a space-separated string
    # FUNCTIONS_ARG=""
    # for element in "${CURRENT_FUNCTIONS_SLICE[@]}"; do
    #     FUNCTIONS_ARG+="$element "
    # done
    # FUNCTIONS_ARG=${FUNCTIONS_ARG%?}
    FUNCTIONS_ARG=$(printf " %s" "${CURRENT_FUNCTIONS_SLICE[@]}")
    # Remove the leading space
    FUNCTIONS_ARG=${FUNCTIONS_ARG:1}

    echo
    echo "Dispatching run #$((i + 1)) with functions: [${FUNCTIONS_ARG}]"

    # Execute the target script in the background
    # The '&' at the end puts the command into a background process
    "$TARGET_SCRIPT" "$FUNCTIONS_ARG" &

done

wait
# Wait for all background jobs to finish before the dispatcher script exits
echo "------------------------------------------------"
echo "All parallel jobs dispatched. Waiting for them to complete..."
wait

echo "All jobs finished."
