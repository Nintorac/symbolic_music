FROM python:3.11-slim

# Set working directory
WORKDIR /workspace

# Copy the aria-utils submodule
COPY aria-utils/ /workspace/aria-utils/

# Install aria-utils
WORKDIR /workspace/aria-utils
RUN pip install -e .

# Copy tokenization script
COPY tokenize_test_files.py /workspace/

# Create output directory
RUN mkdir -p /workspace/output

# Set the default command
CMD ["python", "/workspace/tokenize_test_files.py"]