require 'minitest/autorun'
require_relative '../lib/procfile'

class TestProcfile < Minitest::Test
  def test_empty_parse_procfile
    assert_equal({}, Procfile.parse("").processes)
  end

  def test_valid_parse_procfile
    procfile = Procfile.parse("web: rails s")
    assert_equal({ "web" => "rails s" }, procfile.processes)
  end

  def test_multiple_valid_parse_procfile
    procfile = Procfile.parse("web: rails s\nworker: rake sidekiq")
    assert_equal({ 
      "web" => "rails s",
      "worker" => "rake sidekiq"
    }, procfile.processes)
  end

  def test_to_launch_single_web_process
    procfile = Procfile.new
    procfile.insert("web", "web_command")

    launch = procfile.to_launch

    assert_equal({
      labels: [],
      processes: [{
        type: "web",
        command: ["bash", "-c"],
        args: ["web_command"],
        default: true,
        working_directory: "app"
      }],
      slices: []
    }, launch)
  end

  def test_to_launch_single_non_web_process
    procfile = Procfile.new
    procfile.insert("worker", "worker_command")

    launch = procfile.to_launch

    assert_equal({
      labels: [],
      processes: [{
        type: "worker",
        command: ["bash", "-c"],
        args: ["worker_command"],
        default: true,
        working_directory: "app"
      }],
      slices: []
    }, launch)
  end
end
