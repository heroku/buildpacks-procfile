require 'minitest/autorun'
require_relative '../lib/buildpack'

class TestProcfileBuildpack < Minitest::Test
  def setup
    @buildpack = ProcfileBuildpack.new
    @fixtures_dir = File.join(File.dirname(__FILE__), 'fixtures')
  end

  def test_valid_detect
    app_dir = File.join(@fixtures_dir, 'web_and_worker_procfile')
    assert @buildpack.detect(app_dir)
  end

  def test_missing_procfile_detect
    app_dir = File.join(@fixtures_dir, 'missing_procfile')
    refute @buildpack.detect(app_dir)
  end

  def test_build_with_valid_procfile
    app_dir = File.join(@fixtures_dir, 'web_and_worker_procfile')
    result = @buildpack.build(app_dir)

    assert_equal 2, result[:processes].size
    assert_equal 'web', result[:processes].first[:type]
    assert_equal 'worker', result[:processes].last[:type]
  end

  def test_build_with_empty_procfile
    app_dir = File.join(@fixtures_dir, 'empty_procfile')
    result = @buildpack.build(app_dir)

    assert_equal [], result[:processes]
  end
end
