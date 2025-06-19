require 'yaml'

class Procfile
  attr_reader :processes

  def initialize
    @processes = {}
  end

  def empty?
    @processes.empty?
  end

  def insert(key, value)
    @processes[key.to_s] = value.to_s
  end

  def self.parse(content)
    procfile = new
    
    # Convert CRLF to LF
    content = content.gsub(/\r\n?/, "\n")
    content = content.sub(/\n*\z/, "\n")

    content.each_line do |line|
      if line =~ /^[[:space:]]*([a-zA-Z0-9_-]+):?\\s+(.*)\\s*$/
        procfile.insert($1, $2)
      end
    end

    procfile
  end

  def to_launch
    processes = @processes.map do |key, value|
      {
        type: key,
        command: ['bash', '-c'],
        args: [value],
        default: key == 'web',
        working_directory: 'app'
      }
    end

    # If there's only one process, make it default
    if processes.length == 1
      processes.first[:default] = true
    end

    {
      labels: [],
      processes: processes,
      slices: []
    }
  end
end
